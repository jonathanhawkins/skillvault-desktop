import {
  getOptimizationStatus,
  applyAllOptimizations,
  resetAllOptimizations,
  setAlwaysThinking,
  writeZshrcBlock,
  removeZshrcBlock,
  listProjectsWithLaunchInfo,
  writeLaunchScript,
  removeLaunchScript,
  detectTerminals,
  launchTerminalWithClaude,
  saveOptimizationProfile,
  loadOptimizationProfile,
} from '../lib/api';
import type { OptimizationProfile, OptimizationStatus, DetectedTerminal, ProjectWithLaunchScript } from '../lib/types';
import { showToast } from '../components/toast';

// Current profile state (in-memory, not persisted until applied)
let profile: OptimizationProfile = {
  max_thinking_tokens: 50000,
  autocompact_pct: 45,
  disable_adaptive_thinking: true,
  always_thinking_enabled: true,
  auto_background_tasks: false,
  no_flicker: false,
  skip_permissions: false,
  use_tmux: false,
  experimental_agent_teams: false,
  task_list_id: '',
  extra_cli_args: '',
  model: '',
  effort_level: '',
};

// Model options shown in the picker. Aliases first, then pinned version IDs.
// See https://code.claude.com/docs/en/model-config
const MODEL_OPTIONS: Array<{ value: string; label: string }> = [
  { value: '', label: 'Default (no override)' },
  { value: 'opus', label: 'opus — latest Opus' },
  { value: 'sonnet', label: 'sonnet — latest Sonnet' },
  { value: 'haiku', label: 'haiku — latest Haiku' },
  { value: 'opusplan', label: 'opusplan — Opus in plan, Sonnet to execute' },
  { value: 'best', label: 'best — most capable available' },
  { value: 'opus[1m]', label: 'opus[1m] — 1M context' },
  { value: 'sonnet[1m]', label: 'sonnet[1m] — 1M context' },
  { value: 'claude-opus-4-7', label: 'claude-opus-4-7' },
  { value: 'claude-opus-4-6', label: 'claude-opus-4-6' },
  { value: 'claude-sonnet-4-6', label: 'claude-sonnet-4-6' },
  { value: 'claude-haiku-4-5-20251001', label: 'claude-haiku-4-5-20251001' },
];

// Effort levels supported on Opus 4.6+ / Sonnet 4.6+.
// See https://code.claude.com/docs/en/settings and /docs/en/model-config#adjust-effort-level
const EFFORT_OPTIONS: Array<{ value: string; label: string }> = [
  { value: '', label: 'Default (no override)' },
  { value: 'low', label: 'low — fastest, minimal reasoning' },
  { value: 'medium', label: 'medium — balanced' },
  { value: 'high', label: 'high — deeper reasoning' },
  { value: 'max', label: 'max — deepest (Opus 4.6 only)' },
  { value: 'auto', label: 'auto — reset to model default' },
];

let status: OptimizationStatus | null = null;
let terminals: DetectedTerminal[] = [];
let projects: ProjectWithLaunchScript[] = [];
let selectedProjectIdx = 0;
let selectedTerminalIdx = 0;
let confirmingReset = false;
let previewGeneration = 0;
let previewDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let saveDebounceTimer: ReturnType<typeof setTimeout> | null = null;

/** Debounced save of profile to disk */
function persistProfile() {
  if (saveDebounceTimer) clearTimeout(saveDebounceTimer);
  saveDebounceTimer = setTimeout(() => {
    saveOptimizationProfile(profile).catch(() => {});
  }, 500);
}

/** Escape HTML to prevent XSS when interpolating into innerHTML */
function esc(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/** Build env export lines from profile (matches Rust build_env_export_block) */
function buildEnvExportBlock(p: OptimizationProfile): string {
  const lines: string[] = [];
  if (p.disable_adaptive_thinking) lines.push('export CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1');
  if (p.auto_background_tasks) lines.push('export CLAUDE_AUTO_BACKGROUND_TASKS=1');
  if (p.no_flicker) lines.push('export CLAUDE_CODE_NO_FLICKER=1');
  if (p.experimental_agent_teams) lines.push('export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1');
  lines.push(`export MAX_THINKING_TOKENS=${p.max_thinking_tokens}`);
  lines.push(`export CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=${p.autocompact_pct}`);
  if (p.model) lines.push(`export ANTHROPIC_MODEL=${p.model}`);
  if (p.effort_level) lines.push(`export CLAUDE_CODE_EFFORT_LEVEL=${p.effort_level}`);
  return lines.join('\n');
}

/** Build inline env var string from profile (matches Rust build_env_inline) */
function buildEnvInline(p: OptimizationProfile): string {
  const parts: string[] = [];
  if (p.disable_adaptive_thinking) parts.push('CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1');
  if (p.auto_background_tasks) parts.push('CLAUDE_AUTO_BACKGROUND_TASKS=1');
  if (p.no_flicker) parts.push('CLAUDE_CODE_NO_FLICKER=1');
  if (p.experimental_agent_teams) parts.push('CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1');
  if (p.task_list_id) parts.push(`CLAUDE_CODE_TASK_LIST_ID=${p.task_list_id}`);
  parts.push(`MAX_THINKING_TOKENS=${p.max_thinking_tokens}`);
  parts.push(`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=${p.autocompact_pct}`);
  if (p.model) parts.push(`ANTHROPIC_MODEL=${p.model}`);
  if (p.effort_level) parts.push(`CLAUDE_CODE_EFFORT_LEVEL=${p.effort_level}`);
  return parts.join(' ');
}

/** Generate shell profile block preview (matches Rust generate_block) */
function previewShellBlock(p: OptimizationProfile): string {
  const exports = buildEnvExportBlock(p);
  return `# --- SkillVault Claude Optimizer (start) ---\n# Applied by SkillVault Desktop\n${exports}\n# --- SkillVault Claude Optimizer (end) ---`;
}

/** Generate launch script preview (matches Rust generate_script) */
function previewScript(projectName: string, p: OptimizationProfile): string {
  const envInline = buildEnvInline(p);
  const safeName = projectName.replace(/\n/g, ' ').replace(/\r/g, ' ');
  const cliArgs = p.extra_cli_args ? ` ${p.extra_cli_args.trim()}` : '';
  return `#!/bin/bash\n# Generated by SkillVault Desktop Claude Optimizer\n# Project: ${safeName}\n\ncd "$(dirname "$0")" || exit 1\n${envInline} claude${cliArgs}\n`;
}

/** Build the full launch command for the terminal launcher preview */
function buildLaunchCommand(p: OptimizationProfile, projectPath: string, projectName?: string): string {
  const envParts = buildEnvInline(p);
  // Auto-set task list ID from project name if empty and agent teams is on
  let taskEnv = '';
  if (!p.task_list_id && projectName && p.experimental_agent_teams) {
    taskEnv = ` CLAUDE_CODE_TASK_LIST_ID=${projectName}`;
  }
  // Auto-add CLI flags from toggles
  let cliArgs = '';
  if (p.skip_permissions && !(p.extra_cli_args || '').includes('--dangerously-skip-permissions')) {
    cliArgs += ' --dangerously-skip-permissions';
  }
  if (p.experimental_agent_teams && !(p.extra_cli_args || '').includes('--teammate-mode')) {
    cliArgs += ' --teammate-mode tmux';
  }
  if (p.extra_cli_args) {
    cliArgs += ` ${p.extra_cli_args.trim()}`;
  }
  const fullEnv = `${taskEnv} ${envParts}`.trim();
  const claudeCmd = `claude${cliArgs}`;

  if (p.use_tmux) {
    const sessionName = projectName || 'claude';
    return `cd '${projectPath}' && tmux new-session -A -s ${sessionName} '${fullEnv} ${claudeCmd}'`;
  }
  return `cd '${projectPath}' && ${fullEnv} ${claudeCmd}`;
}

function scoreColor(score: number): string {
  if (score === 4) return 'var(--success)';
  if (score >= 2) return 'var(--warning)';
  return 'var(--error)';
}

function scoreDots(score: number): string {
  return Array.from({ length: 4 }, (_, i) =>
    `<span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${i < score ? scoreColor(score) : 'var(--border)'};margin-right:4px"></span>`
  ).join('');
}

function scoreLabel(score: number): string {
  if (score === 4) return 'All optimizations active';
  if (score === 0) return 'No optimizations active';
  return `${score} of 4 optimizations active`;
}

function renderToggle(id: string, checked: boolean, label: string, description: string, detail: string): string {
  return `
    <div class="opt-card">
      <div style="display:flex;align-items:center;justify-content:space-between;gap:12px">
        <div style="flex:1">
          <div style="font-size:14px;color:var(--text-primary);font-weight:500">${label}</div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:4px">${description}</div>
          <div style="font-size:11px;color:var(--text-faint);margin-top:4px;font-family:'Geist Mono',monospace">${detail}</div>
        </div>
        <label class="opt-toggle">
          <input type="checkbox" id="${id}" ${checked ? 'checked' : ''}>
          <span class="opt-toggle-slider"></span>
        </label>
      </div>
    </div>
  `;
}

function renderSelect(id: string, current: string, options: Array<{ value: string; label: string }>, label: string, description: string, detail: string): string {
  const opts = options.map(o =>
    `<option value="${esc(o.value)}"${o.value === current ? ' selected' : ''}>${esc(o.label)}</option>`
  ).join('');
  return `
    <div class="opt-card">
      <div style="font-size:14px;color:var(--text-primary);font-weight:500">${label}</div>
      <div style="font-size:12px;color:var(--text-muted);margin-top:4px">${description}</div>
      <div style="font-size:11px;color:var(--text-faint);margin-top:4px;font-family:'Geist Mono',monospace">${detail}</div>
      <select id="${id}" class="opt-select" style="margin-top:10px;width:100%">${opts}</select>
    </div>
  `;
}

function renderSlider(id: string, value: number, min: number, max: number, step: number, label: string, description: string, detail: string, enabled: boolean): string {
  return `
    <div class="opt-card">
      <div style="display:flex;align-items:center;justify-content:space-between;gap:12px">
        <div style="flex:1">
          <div style="font-size:14px;color:var(--text-primary);font-weight:500">${label}</div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:4px">${description}</div>
          <div style="font-size:11px;color:var(--text-faint);margin-top:4px;font-family:'Geist Mono',monospace">${detail}</div>
        </div>
        <span id="${id}-value" style="font-family:'Geist Mono',monospace;font-size:14px;color:${enabled ? 'var(--accent)' : 'var(--text-faint)'};min-width:60px;text-align:right">${value.toLocaleString()}</span>
      </div>
      <input type="range" id="${id}" class="opt-range" min="${min}" max="${max}" step="${step}" value="${value}" ${enabled ? '' : 'disabled'} style="margin-top:10px;width:100%">
    </div>
  `;
}

export async function renderOptimize() {
  const content = document.getElementById('content');
  if (!content) return;

  // Reset ephemeral state on view mount
  confirmingReset = false;
  selectedProjectIdx = 0;
  selectedTerminalIdx = 0;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Optimize Claude Code</h1>
      </div>
    </div>
    <div style="display:flex;align-items:center;gap:12px;color:var(--text-muted);font-size:13px">
      <div class="spinner" style="width:16px;height:16px"></div>
      Loading optimization status...
    </div>
  `;

  // Load data in parallel
  try {
    const [s, t, p, saved] = await Promise.all([
      getOptimizationStatus(),
      detectTerminals(),
      listProjectsWithLaunchInfo(),
      loadOptimizationProfile(),
    ]);
    status = s;
    terminals = t;
    projects = p;

    // Load saved profile preferences first (toggles, text inputs)
    if (saved) {
      profile = { ...profile, ...saved };
    }

    // Then overlay live-detected values for the core 4 settings
    profile.always_thinking_enabled = s.always_thinking_enabled;
    if (s.max_thinking_tokens) profile.max_thinking_tokens = parseInt(s.max_thinking_tokens, 10) || 50000;
    if (s.autocompact_pct_override) profile.autocompact_pct = parseInt(s.autocompact_pct_override, 10) || 45;
    if (s.disable_adaptive_thinking !== null) {
      profile.disable_adaptive_thinking = s.disable_adaptive_thinking === '1';
    }
  } catch (e: any) {
    content.innerHTML = `
      <div class="view-header"><div class="view-header-title"><h1 class="h1">Optimize Claude Code</h1></div></div>
      <div style="color:var(--error);font-size:13px">Failed to load: ${e}</div>
    `;
    return;
  }

  renderView(content);
}

function renderView(content: HTMLElement) {
  if (!status) return;
  const s = status;

  const shellFile = s.shell_profile_path.split('/').pop() || s.shell_profile_path;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Optimize Claude Code</h1>
      </div>
      <div style="display:flex;gap:8px">
        <button class="btn btn--primary btn--sm" id="apply-all-btn">1-Click Fix All</button>
        <button class="btn btn--sm" id="reset-all-btn" style="color:var(--error);border-color:rgba(239,68,68,0.3)">${confirmingReset ? 'Confirm Reset' : 'Reset All'}</button>
      </div>
    </div>

    <!-- Score banner -->
    <div style="display:flex;align-items:center;gap:12px;padding:14px 16px;background:${s.optimization_score === 4 ? 'rgba(34,197,94,0.08)' : s.optimization_score > 0 ? 'rgba(234,179,8,0.08)' : 'rgba(239,68,68,0.08)'};border:1px solid ${s.optimization_score === 4 ? 'rgba(34,197,94,0.2)' : s.optimization_score > 0 ? 'rgba(234,179,8,0.2)' : 'rgba(239,68,68,0.2)'};border-radius:8px;margin-bottom:24px">
      <div>${scoreDots(s.optimization_score)}</div>
      <div style="font-size:13px;color:${scoreColor(s.optimization_score)};font-weight:500">${scoreLabel(s.optimization_score)}</div>
    </div>

    <!-- Tier 1: Settings + Shell Profile -->
    <div class="opt-section">
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px">
        <div class="opt-section-label" style="margin-bottom:0">Settings &amp; Environment</div>
        <button class="btn btn--sm" id="open-settings-btn" style="font-size:11px;padding:4px 10px;display:inline-flex;align-items:center;gap:5px">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
          Open settings.json
        </button>
      </div>
      <p style="font-size:12px;color:var(--text-muted);margin-bottom:16px">Configure Claude Code's settings.json and shell environment variables for optimal performance.${!s.settings_json_exists ? ' <span style="color:var(--warning)">settings.json does not exist yet — toggling a setting will create it.</span>' : ''}</p>

      ${renderToggle('toggle-thinking', profile.always_thinking_enabled,
        'Always Extended Thinking',
        'Forces Claude Code to use extended thinking on every response, preventing performance degradation.',
        '~/.claude/settings.json → alwaysThinkingEnabled'
      )}

      ${renderToggle('toggle-adaptive', profile.disable_adaptive_thinking,
        'Disable Adaptive Thinking',
        'Prevents Claude from dynamically reducing thinking budget, which causes progressively shorter responses.',
        'CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1'
      )}

      ${renderSlider('slider-tokens', profile.max_thinking_tokens, 5000, 160000, 5000,
        'Max Thinking Tokens',
        'Sets the thinking token budget per response. Higher values give Claude more reasoning space.',
        'MAX_THINKING_TOKENS',
        true
      )}

      ${renderSlider('slider-autocompact', profile.autocompact_pct, 10, 95, 5,
        'Autocompact Threshold (%)',
        'Controls when Claude compacts conversation context. Lower values trigger compaction sooner, keeping more working memory.',
        'CLAUDE_AUTOCOMPACT_PCT_OVERRIDE',
        true
      )}

      ${renderSelect('select-model', profile.model, MODEL_OPTIONS,
        'Model',
        'Which Claude model Claude Code uses. Aliases track the latest release; pinned IDs (like claude-opus-4-7) lock to a specific version.',
        'ANTHROPIC_MODEL'
      )}

      ${renderSelect('select-effort', profile.effort_level, EFFORT_OPTIONS,
        'Effort Level',
        'Reasoning depth for Opus 4.6+ / Sonnet 4.6+ models. Higher levels reason longer but cost more; max is Opus 4.6 only.',
        'CLAUDE_CODE_EFFORT_LEVEL'
      )}

      ${renderToggle('toggle-background', profile.auto_background_tasks,
        'Auto Background Tasks',
        'Allow Claude to automatically run tasks in the background for parallel work.',
        'CLAUDE_AUTO_BACKGROUND_TASKS=1'
      )}

      ${renderToggle('toggle-noflicker', profile.no_flicker,
        'No Flicker Mode',
        'Reduces terminal flickering during output by buffering screen updates. Improves visual stability.',
        'CLAUDE_CODE_NO_FLICKER=1'
      )}

      ${renderToggle('toggle-skip-perms', profile.skip_permissions,
        'Skip Permissions',
        'Run Claude Code without permission prompts. Use with caution — grants full autonomy.',
        '--dangerously-skip-permissions'
      )}

      ${renderToggle('toggle-tmux', profile.use_tmux,
        'Use tmux Sessions',
        'Launch Claude Code inside a named tmux session. Auto-increments names when sessions already exist.',
        'tmux new-session -A -s &lt;project-name&gt;'
      )}

      ${renderToggle('toggle-teams', profile.experimental_agent_teams,
        'Experimental Agent Teams',
        'Enable multi-agent team coordination for parallel work.',
        'CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1'
      )}

      <div class="opt-card">
        <div style="font-size:14px;color:var(--text-primary);font-weight:500">Task List ID</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Set a default task list ID for project context.</div>
        <div style="font-size:11px;color:var(--text-faint);margin-top:4px;font-family:'Geist Mono',monospace">CLAUDE_CODE_TASK_LIST_ID</div>
        <input type="text" id="input-task-list" value="${esc(profile.task_list_id)}" placeholder="e.g. skill-vault" style="margin-top:10px;width:100%;padding:8px 10px;background:var(--bg-primary);border:1px solid var(--border);border-radius:6px;color:var(--text-primary);font-family:'Geist Mono',monospace;font-size:12px">
      </div>

      <div class="opt-card">
        <div style="font-size:14px;color:var(--text-primary);font-weight:500">Extra CLI Arguments</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Additional flags passed to the <span style="font-family:'Geist Mono',monospace">claude</span> command in launch scripts.</div>
        <input type="text" id="input-cli-args" value="${esc(profile.extra_cli_args)}" placeholder="e.g. --dangerously-skip-permissions --teammate-mode tmux" style="margin-top:10px;width:100%;padding:8px 10px;background:var(--bg-primary);border:1px solid var(--border);border-radius:6px;color:var(--text-primary);font-family:'Geist Mono',monospace;font-size:12px">
      </div>
    </div>

    <!-- Tier 1: Shell Profile Apply -->
    <div class="opt-section">
      <div class="opt-section-label">Shell Profile</div>
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px">
        <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${s.shell_block_exists ? 'var(--success)' : 'var(--text-faint)'}"></span>
        <span style="font-size:12px;color:${s.shell_block_exists ? 'var(--success)' : 'var(--text-muted)'}">${s.shell_block_exists ? 'Active' : 'Not applied'}</span>
        <span style="font-size:11px;color:var(--text-faint);font-family:'Geist Mono',monospace;margin-left:auto">${esc(s.shell_profile_path)}</span>
      </div>
      <div id="shell-preview" style="background:var(--bg-primary);border:1px solid var(--border);border-radius:6px;padding:12px;font-family:'Geist Mono',monospace;font-size:11px;color:var(--text-secondary);white-space:pre;overflow:auto;max-height:160px;margin-bottom:12px">Loading preview...</div>
      <div style="display:flex;gap:8px;align-items:center">
        <button class="btn btn--primary btn--sm" id="apply-shell-btn">${s.shell_block_exists ? 'Update' : 'Apply to'} ~/${shellFile}</button>
        ${s.shell_block_exists ? '<button class="btn btn--sm" id="remove-shell-btn" style="color:var(--error);border-color:rgba(239,68,68,0.3)">Remove</button>' : ''}
        <span style="font-size:11px;color:var(--text-faint);margin-left:auto">⚠ Requires terminal restart</span>
      </div>
    </div>

    <!-- Tier 2: Project Launch Scripts -->
    <div class="opt-section">
      <div class="opt-section-label">Project Launch Scripts</div>
      <p style="font-size:12px;color:var(--text-muted);margin-bottom:12px">Generate a <span style="font-family:'Geist Mono',monospace">start-claude.sh</span> script for your projects with optimized environment variables.</p>
      ${projects.length === 0
        ? '<div style="font-size:12px;color:var(--text-faint);padding:16px;border:1px solid var(--border);border-radius:6px;text-align:center">No Claude Code projects detected</div>'
        : `
          <div id="project-list" style="display:flex;flex-direction:column;gap:4px;margin-bottom:12px;max-height:140px;overflow-y:auto">
            ${projects.map((p, i) => `
              <div class="opt-project-row${i === selectedProjectIdx ? ' opt-project-row--active' : ''}" data-project-idx="${i}">
                <span style="font-size:13px;color:var(--text-primary);font-weight:500">${esc(p.name)}</span>
                <span style="font-size:11px;color:var(--text-faint);font-family:'Geist Mono',monospace;flex:1;text-overflow:ellipsis;overflow:hidden;white-space:nowrap;margin-left:8px">${esc(p.path)}</span>
                ${p.has_launch_script ? '<span style="font-size:11px;color:var(--success)">✓ script</span>' : ''}
              </div>
            `).join('')}
          </div>
          <div id="script-preview" style="background:var(--bg-primary);border:1px solid var(--border);border-radius:6px;padding:12px;font-family:'Geist Mono',monospace;font-size:11px;color:var(--text-secondary);white-space:pre;overflow:auto;max-height:160px;margin-bottom:12px">Loading preview...</div>
          <div style="display:flex;gap:8px">
            <button class="btn btn--primary btn--sm" id="create-script-btn">${projects[selectedProjectIdx]?.has_launch_script ? 'Update' : 'Create'} start-claude.sh</button>
            ${projects[selectedProjectIdx]?.has_launch_script ? '<button class="btn btn--sm" id="remove-script-btn" style="color:var(--error);border-color:rgba(239,68,68,0.3)">Remove</button>' : ''}
          </div>
        `
      }
    </div>

    <!-- Tier 3: Terminal Launcher -->
    <div class="opt-section">
      <div class="opt-section-label">Terminal Launcher</div>
      <p style="font-size:12px;color:var(--text-muted);margin-bottom:12px">Launch Claude Code directly with optimized settings in your preferred terminal.</p>
      ${projects.length === 0
        ? '<div style="font-size:12px;color:var(--text-faint);padding:16px;border:1px solid var(--border);border-radius:6px;text-align:center">No projects available</div>'
        : `
          <div style="display:flex;gap:12px;margin-bottom:16px;flex-wrap:wrap">
            <div style="flex:1;min-width:200px">
              <label style="font-size:11px;color:var(--text-faint);font-family:'Geist Mono',monospace;letter-spacing:0.5px;text-transform:uppercase;display:block;margin-bottom:6px">Project</label>
              <select id="launch-project" class="opt-select">
                ${projects.map((p, i) => `<option value="${i}" ${i === selectedProjectIdx ? 'selected' : ''}>${esc(p.name)}</option>`).join('')}
              </select>
            </div>
            <div style="flex:1;min-width:200px">
              <label style="font-size:11px;color:var(--text-faint);font-family:'Geist Mono',monospace;letter-spacing:0.5px;text-transform:uppercase;display:block;margin-bottom:6px">Terminal</label>
              <select id="launch-terminal" class="opt-select">
                ${terminals.map((t, i) => `<option value="${i}" ${i === selectedTerminalIdx ? 'selected' : ''}>${esc(t.name)}</option>`).join('')}
              </select>
            </div>
          </div>
          <div style="margin-bottom:12px">
            <label style="font-size:11px;color:var(--text-faint);font-family:'Geist Mono',monospace;letter-spacing:0.5px;text-transform:uppercase;display:block;margin-bottom:6px">Command</label>
            <div id="launch-preview" style="background:var(--bg-primary);border:1px solid var(--border);border-radius:6px;padding:12px;font-family:'Geist Mono',monospace;font-size:11px;color:var(--text-secondary);white-space:pre-wrap;word-break:break-all;overflow:auto;max-height:80px">${esc(buildLaunchCommand(profile, projects[selectedProjectIdx]?.path || '', projects[selectedProjectIdx]?.name))}</div>
          </div>
          <button class="btn btn--lg" id="launch-btn" style="width:100%;background:var(--accent);color:#fff;border:none;font-size:14px;font-weight:600;padding:14px;border-radius:8px;cursor:pointer;transition:background 0.15s">
            Launch Claude Code
          </button>
          ${terminals[selectedTerminalIdx]?.name === 'Warp' || terminals[selectedTerminalIdx]?.name === 'Hyper'
            ? '<div style="font-size:11px;color:var(--warning);margin-top:8px">⚠ This terminal has limited scripting. The launch command will be copied to your clipboard.</div>'
            : ''
          }
        `
      }
    </div>

    <!-- Status footer -->
    <div style="display:flex;gap:24px;padding:12px 0;font-size:11px;color:var(--text-faint);border-top:1px solid var(--border);margin-top:8px">
      <span style="font-family:'Geist Mono',monospace">settings.json ${s.settings_json_exists ? '✓' : '✗'}</span>
      <span style="font-family:'Geist Mono',monospace">${shellFile} ${s.shell_block_exists ? '✓' : '✗'}</span>
      <span style="font-family:'Geist Mono',monospace">${terminals.length} terminal${terminals.length !== 1 ? 's' : ''} detected</span>
    </div>
  `;

  // Load previews
  loadPreviews(content);

  // Bind events
  bindEvents(content);
}

function debouncedLoadPreviews(content: HTMLElement) {
  if (previewDebounceTimer) clearTimeout(previewDebounceTimer);
  previewDebounceTimer = setTimeout(() => loadPreviews(content), 150);
  persistProfile();
}

function loadPreviews(content: HTMLElement) {
  ++previewGeneration;

  // Shell block preview (pure computation, no IPC)
  const shellPreview = content.querySelector('#shell-preview');
  if (shellPreview) {
    shellPreview.textContent = previewShellBlock(profile);
  }

  // Launch script preview (pure computation, no IPC)
  const scriptPreview = content.querySelector('#script-preview');
  if (scriptPreview && projects[selectedProjectIdx]) {
    scriptPreview.textContent = previewScript(projects[selectedProjectIdx].name, profile);
  }

  // Terminal launcher command preview
  const launchPreview = content.querySelector('#launch-preview');
  if (launchPreview && projects[selectedProjectIdx]) {
    launchPreview.textContent = buildLaunchCommand(profile, projects[selectedProjectIdx].path, projects[selectedProjectIdx].name);
  }
}

function bindEvents(content: HTMLElement) {
  // Open settings.json in default editor
  content.querySelector('#open-settings-btn')?.addEventListener('click', async () => {
    try {
      const { invoke } = window.__TAURI__.core;
      await invoke('open_settings_json');
    } catch (e: any) {
      showToast(`Could not open settings.json: ${e}`, 'error');
    }
  });

  // Toggle: always thinking
  content.querySelector('#toggle-thinking')?.addEventListener('change', async (e) => {
    const checked = (e.target as HTMLInputElement).checked;
    profile.always_thinking_enabled = checked;
    try {
      status = await setAlwaysThinking(checked);
      persistProfile();
      showToast(checked ? 'Extended thinking enabled' : 'Extended thinking disabled', 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Toggle: disable adaptive thinking
  content.querySelector('#toggle-adaptive')?.addEventListener('change', (e) => {
    profile.disable_adaptive_thinking = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Slider: max thinking tokens
  content.querySelector('#slider-tokens')?.addEventListener('input', (e) => {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    profile.max_thinking_tokens = val;
    const label = content.querySelector('#slider-tokens-value');
    if (label) label.textContent = val.toLocaleString();
  });
  content.querySelector('#slider-tokens')?.addEventListener('change', () => {
    debouncedLoadPreviews(content);
  });

  // Slider: autocompact pct
  content.querySelector('#slider-autocompact')?.addEventListener('input', (e) => {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    profile.autocompact_pct = val;
    const label = content.querySelector('#slider-autocompact-value');
    if (label) label.textContent = val.toLocaleString();
  });
  content.querySelector('#slider-autocompact')?.addEventListener('change', () => {
    debouncedLoadPreviews(content);
  });

  // Toggle: auto background tasks
  content.querySelector('#toggle-background')?.addEventListener('change', (e) => {
    profile.auto_background_tasks = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Toggle: no flicker
  content.querySelector('#toggle-noflicker')?.addEventListener('change', (e) => {
    profile.no_flicker = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Toggle: skip permissions
  content.querySelector('#toggle-skip-perms')?.addEventListener('change', (e) => {
    profile.skip_permissions = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Toggle: use tmux
  content.querySelector('#toggle-tmux')?.addEventListener('change', (e) => {
    profile.use_tmux = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Toggle: experimental agent teams
  content.querySelector('#toggle-teams')?.addEventListener('change', (e) => {
    profile.experimental_agent_teams = (e.target as HTMLInputElement).checked;
    debouncedLoadPreviews(content);
  });

  // Select: model
  content.querySelector('#select-model')?.addEventListener('change', (e) => {
    profile.model = (e.target as HTMLSelectElement).value;
    debouncedLoadPreviews(content);
  });

  // Select: effort level
  content.querySelector('#select-effort')?.addEventListener('change', (e) => {
    profile.effort_level = (e.target as HTMLSelectElement).value;
    debouncedLoadPreviews(content);
  });

  // Input: task list ID
  content.querySelector('#input-task-list')?.addEventListener('input', (e) => {
    profile.task_list_id = (e.target as HTMLInputElement).value;
    debouncedLoadPreviews(content);
  });

  // Input: extra CLI args
  content.querySelector('#input-cli-args')?.addEventListener('input', (e) => {
    profile.extra_cli_args = (e.target as HTMLInputElement).value;
    debouncedLoadPreviews(content);
  });

  // Apply all
  content.querySelector('#apply-all-btn')?.addEventListener('click', async () => {
    try {
      status = await applyAllOptimizations(profile);
      showToast('All optimizations applied!', 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Reset all
  content.querySelector('#reset-all-btn')?.addEventListener('click', async () => {
    if (!confirmingReset) {
      confirmingReset = true;
      const btn = content.querySelector('#reset-all-btn');
      if (btn) {
        btn.textContent = 'Confirm Reset';
        btn.setAttribute('style', 'color:#fff;background:var(--error);border-color:var(--error)');
      }
      setTimeout(() => {
        confirmingReset = false;
        if (btn) {
          btn.textContent = 'Reset All';
          btn.setAttribute('style', 'color:var(--error);border-color:rgba(239,68,68,0.3)');
        }
      }, 3000);
      return;
    }
    confirmingReset = false;
    try {
      status = await resetAllOptimizations();
      profile = { max_thinking_tokens: 50000, autocompact_pct: 45, disable_adaptive_thinking: true, always_thinking_enabled: true, auto_background_tasks: false, no_flicker: false, skip_permissions: false, use_tmux: false, experimental_agent_teams: false, task_list_id: '', extra_cli_args: '', model: '', effort_level: '' };
      showToast('All optimizations reset', 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Apply shell block
  content.querySelector('#apply-shell-btn')?.addEventListener('click', async () => {
    try {
      status = await writeZshrcBlock(profile);
      showToast('Shell profile updated — restart your terminal', 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Remove shell block
  content.querySelector('#remove-shell-btn')?.addEventListener('click', async () => {
    try {
      status = await removeZshrcBlock();
      showToast('Shell profile block removed', 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Project selection
  content.querySelectorAll('.opt-project-row').forEach((row) => {
    row.addEventListener('click', () => {
      selectedProjectIdx = parseInt((row as HTMLElement).dataset.projectIdx || '0', 10);
      renderView(content);
    });
  });

  // Create/update launch script
  content.querySelector('#create-script-btn')?.addEventListener('click', async () => {
    const p = projects[selectedProjectIdx];
    if (!p) return;
    try {
      await writeLaunchScript(p.path, p.name, profile);
      projects = await listProjectsWithLaunchInfo();
      showToast(`start-claude.sh created in ${p.name}`, 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Remove launch script
  content.querySelector('#remove-script-btn')?.addEventListener('click', async () => {
    const p = projects[selectedProjectIdx];
    if (!p) return;
    try {
      await removeLaunchScript(p.path);
      projects = await listProjectsWithLaunchInfo();
      showToast(`start-claude.sh removed from ${p.name}`, 'success');
      renderView(content);
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
  });

  // Terminal launcher dropdowns
  content.querySelector('#launch-project')?.addEventListener('change', (e) => {
    selectedProjectIdx = parseInt((e.target as HTMLSelectElement).value, 10);
    const lp = content.querySelector('#launch-preview');
    if (lp && projects[selectedProjectIdx]) {
      lp.textContent = buildLaunchCommand(profile, projects[selectedProjectIdx].path, projects[selectedProjectIdx].name);
    }
  });
  content.querySelector('#launch-terminal')?.addEventListener('change', (e) => {
    selectedTerminalIdx = parseInt((e.target as HTMLSelectElement).value, 10);
  });

  // Launch button
  content.querySelector('#launch-btn')?.addEventListener('click', async () => {
    const p = projects[selectedProjectIdx];
    const t = terminals[selectedTerminalIdx];
    if (!p || !t) return;
    const btn = content.querySelector('#launch-btn') as HTMLButtonElement;
    if (btn) {
      btn.disabled = true;
      btn.textContent = 'Launching...';
    }
    try {
      const result = await launchTerminalWithClaude(t.name, p.path, profile);
      showToast(result, 'success');
    } catch (e: any) {
      showToast(`Failed: ${e}`, 'error');
    }
    if (btn) {
      btn.disabled = false;
      btn.textContent = 'Launch Claude Code';
    }
  });
}
