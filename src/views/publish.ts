import { getState, setState } from '../lib/state';
import { scanLocal, packageSkill, publishSkill, getAuthStatus } from '../lib/api';
import { showToast } from '../components/toast';
import { renderSidebar } from '../components/sidebar';
import { navigate } from '../lib/router';
import type { LocalSkill, PackagedSkill } from '../lib/types';

const CATEGORIES = [
  'productivity',
  'development',
  'toolkit',
  'automation',
  'communication',
  'security',
  'testing',
  'documentation',
  'devops',
];

type PublishStep = 'select' | 'metadata' | 'publishing';

let currentStep: PublishStep = 'select';
let selectedSkill: LocalSkill | null = null;
let packaged: PackagedSkill | null = null;

export async function renderPublish() {
  const content = document.getElementById('content');
  if (!content) return;

  // Check auth
  try {
    const auth = await getAuthStatus();
    setState({ authenticated: auth.authenticated });
    if (!auth.authenticated) {
      renderAuthRequired(content);
      return;
    }
  } catch {
    renderAuthRequired(content);
    return;
  }

  // Ensure we have local state
  let state = getState();
  if (!state.localState) {
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Publish Skill</h1>
        </div>
      </div>
      <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
    `;
    try {
      const localState = await scanLocal();
      setState({ localState });
      renderSidebar();
    } catch (e: any) {
      content.innerHTML = `
        <div class="view-header">
          <div class="view-header-title">
            <h1 class="h1">Publish Skill</h1>
          </div>
        </div>
        <div class="empty-state">
          <div class="empty-state-text">Failed to scan local skills: ${esc(e?.toString() || 'Unknown error')}</div>
          <button class="btn btn--sm" id="retry-btn">Retry</button>
        </div>
      `;
      content.querySelector('#retry-btn')?.addEventListener('click', () => renderPublish());
      return;
    }
  }

  switch (currentStep) {
    case 'select':
      renderSelectStep(content);
      break;
    case 'metadata':
      renderMetadataStep(content);
      break;
    case 'publishing':
      renderPublishingStep(content);
      break;
  }
}

function renderAuthRequired(content: HTMLElement) {
  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    <div class="empty-state">
      <svg class="empty-state-icon" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0110 0v4"/>
      </svg>
      <div class="empty-state-text">Authentication required to publish skills.</div>
      <p style="font-size:13px;color:var(--text-secondary);margin-bottom:16px">
        Add your API token in Settings to get started.
      </p>
      <button class="btn btn--primary btn--sm" id="goto-settings-btn">Go to Settings</button>
    </div>
  `;
  content.querySelector('#goto-settings-btn')?.addEventListener('click', () => {
    navigate('settings');
  });
}

function renderSelectStep(content: HTMLElement) {
  const state = getState();
  const ls = state.localState!;

  // Filter to local-only skills (not already published from SkillVault)
  const localSkills = ls.skills.filter((s) => s.source === 'local');

  const stepsHtml = renderSteps('select');

  if (localSkills.length === 0) {
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Publish Skill</h1>
        </div>
      </div>
      ${stepsHtml}
      <div class="empty-state">
        <svg class="empty-state-icon" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        <div class="empty-state-text">No local skills to publish.</div>
        <p style="font-size:13px;color:var(--text-secondary)">
          Skills sourced from SkillVault are already published. Create a new skill in <code style="font-size:12px;background:var(--bg-secondary);padding:2px 6px;border-radius:4px">~/.claude/skills/</code> first.
        </p>
      </div>
    `;
    return;
  }

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <p style="font-size:13px;color:var(--text-secondary);margin-bottom:16px">
      Select a local skill to publish to skillvault.md
    </p>
    <div class="grid">
      ${localSkills
        .map(
          (skill) => `
        <div class="skill-card skill-card--clickable publish-select-card${selectedSkill?.name === skill.name ? ' publish-select-card--selected' : ''}" data-skill="${esc(skill.name)}">
          <div class="skill-card-header">
            <div class="skill-card-name">${esc(skill.name)}</div>
            <span class="skill-card-source skill-card-source--local">local</span>
          </div>
          ${skill.description ? `<div class="skill-card-desc">${esc(skill.description)}</div>` : ''}
          <div class="skill-card-meta">
            <span>${skill.file_count} files</span>
            ${skill.has_scripts ? '<span>scripts</span>' : ''}
            ${skill.has_subagents ? '<span>subagents</span>' : ''}
          </div>
        </div>
      `
        )
        .join('')}
    </div>
    <div style="display:flex;justify-content:flex-end;margin-top:24px">
      <button class="btn btn--primary" id="next-btn"${!selectedSkill ? ' disabled' : ''}>Next</button>
    </div>
  `;

  // Bind card selection
  content.querySelectorAll('.publish-select-card').forEach((card) => {
    card.addEventListener('click', () => {
      const name = (card as HTMLElement).dataset.skill;
      if (!name) return;
      const skill = localSkills.find((s) => s.name === name);
      if (!skill) return;

      selectedSkill = skill;

      // Update selected state visually
      content.querySelectorAll('.publish-select-card').forEach((c) => {
        c.classList.remove('publish-select-card--selected');
      });
      card.classList.add('publish-select-card--selected');

      // Enable Next button
      const nextBtn = content.querySelector('#next-btn') as HTMLButtonElement;
      if (nextBtn) nextBtn.disabled = false;
    });
  });

  // Next button
  content.querySelector('#next-btn')?.addEventListener('click', async () => {
    if (!selectedSkill) return;

    // Package the skill to get metadata preview
    const nextBtn = content.querySelector('#next-btn') as HTMLButtonElement;
    if (nextBtn) {
      nextBtn.disabled = true;
      nextBtn.textContent = 'Packaging...';
    }

    try {
      packaged = await packageSkill(selectedSkill.name);
      currentStep = 'metadata';
      renderPublish();
    } catch (e: any) {
      showToast(`Failed to package skill: ${e}`, 'error');
      if (nextBtn) {
        nextBtn.disabled = false;
        nextBtn.textContent = 'Next';
      }
    }
  });
}

function renderMetadataStep(content: HTMLElement) {
  if (!selectedSkill || !packaged) {
    currentStep = 'select';
    renderPublish();
    return;
  }

  const stepsHtml = renderSteps('metadata');

  // Derive default display name from skill name (kebab-case to Title Case)
  const defaultDisplayName = selectedSkill.name
    .split(/[-_]/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');

  const defaultTagline = packaged.description || '';

  const sizeStr = formatBytes(packaged.size_bytes);

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <div style="max-width:560px">
      <div style="margin-bottom:24px;padding:16px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:8px">
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:8px">
          <span style="font-weight:600;font-size:14px;color:var(--text-primary)">${esc(selectedSkill.name)}</span>
          <span style="font-size:12px;color:var(--text-muted)">${packaged.file_count} files</span>
          <span style="font-size:12px;color:var(--text-muted)">${sizeStr}</span>
        </div>
      </div>

      <div class="publish-form">
        <div class="publish-field">
          <label class="settings-label" for="pub-display-name">Display Name</label>
          <input class="settings-input" id="pub-display-name" type="text" value="${esc(defaultDisplayName)}" placeholder="My Awesome Skill">
        </div>
        <div class="publish-field">
          <label class="settings-label" for="pub-tagline">Tagline</label>
          <input class="settings-input" id="pub-tagline" type="text" value="${esc(defaultTagline)}" placeholder="A short description of what this skill does">
        </div>
        <div class="publish-field">
          <label class="settings-label" for="pub-category">Category</label>
          <select class="settings-input" id="pub-category">
            ${CATEGORIES.map(
              (cat) =>
                `<option value="${cat}"${cat === 'development' ? ' selected' : ''}>${cat.charAt(0).toUpperCase() + cat.slice(1)}</option>`
            ).join('')}
          </select>
        </div>
        <div class="publish-field">
          <label class="settings-label" for="pub-version">Version</label>
          <input class="settings-input" id="pub-version" type="text" value="1.0.0" placeholder="1.0.0">
        </div>
      </div>

      <div style="display:flex;gap:12px;justify-content:flex-end;margin-top:24px">
        <button class="btn" id="back-btn">Back</button>
        <button class="btn btn--primary" id="publish-btn">Publish</button>
      </div>
    </div>
  `;

  // Back button
  content.querySelector('#back-btn')?.addEventListener('click', () => {
    currentStep = 'select';
    renderPublish();
  });

  // Publish button
  content.querySelector('#publish-btn')?.addEventListener('click', async () => {
    const displayName = (content.querySelector('#pub-display-name') as HTMLInputElement).value.trim();
    const tagline = (content.querySelector('#pub-tagline') as HTMLInputElement).value.trim();
    const category = (content.querySelector('#pub-category') as HTMLSelectElement).value;
    const version = (content.querySelector('#pub-version') as HTMLInputElement).value.trim();

    if (!displayName) {
      showToast('Display name is required', 'error');
      return;
    }
    if (!version || !/^\d+\.\d+\.\d+/.test(version)) {
      showToast('Version must be in semver format (e.g. 1.0.0)', 'error');
      return;
    }

    currentStep = 'publishing';
    renderPublishingStep(content, displayName, tagline, category, version);
  });
}

async function renderPublishingStep(
  content: HTMLElement,
  displayName?: string,
  tagline?: string,
  category?: string,
  version?: string
) {
  if (!selectedSkill) {
    currentStep = 'select';
    renderPublish();
    return;
  }

  const stepsHtml = renderSteps('publishing');

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <div style="display:flex;flex-direction:column;align-items:center;padding:48px 0;gap:16px">
      <div class="spinner"></div>
      <div id="publish-status" style="font-size:14px;color:var(--text-secondary)">Packaging...</div>
    </div>
  `;

  if (!displayName || !tagline === undefined || !category || !version) {
    currentStep = 'metadata';
    renderPublish();
    return;
  }

  const statusEl = content.querySelector('#publish-status');

  try {
    if (statusEl) statusEl.textContent = 'Uploading to skillvault.md...';

    const result = await publishSkill(
      selectedSkill.name,
      displayName,
      tagline ?? '',
      category,
      version
    );

    // Success state
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Publish Skill</h1>
        </div>
      </div>
      ${stepsHtml}
      <div style="display:flex;flex-direction:column;align-items:center;padding:48px 0;gap:16px">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
          <polyline points="22 4 12 14.01 9 11.01"/>
        </svg>
        <div style="font-size:16px;font-weight:600;color:var(--text-primary)">Published!</div>
        <div style="font-size:13px;color:var(--text-secondary)">${esc(result)}</div>
        <div style="display:flex;gap:12px;margin-top:16px">
          <button class="btn" id="view-on-sv-btn">View on skillvault.md</button>
          <button class="btn btn--primary" id="done-btn">Done</button>
        </div>
      </div>
    `;

    content.querySelector('#view-on-sv-btn')?.addEventListener('click', async () => {
      try {
        const { open } = window.__TAURI__.shell;
        await open(`https://skillvault.md/packages`);
      } catch {
        // ignore
      }
    });

    content.querySelector('#done-btn')?.addEventListener('click', () => {
      currentStep = 'select';
      selectedSkill = null;
      packaged = null;
      navigate('installed');
    });
  } catch (e: any) {
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Publish Skill</h1>
        </div>
      </div>
      ${stepsHtml}
      <div style="display:flex;flex-direction:column;align-items:center;padding:48px 0;gap:16px">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <line x1="15" y1="9" x2="9" y2="15"/>
          <line x1="9" y1="9" x2="15" y2="15"/>
        </svg>
        <div style="font-size:16px;font-weight:600;color:#ef4444">Publishing Failed</div>
        <div style="font-size:13px;color:var(--text-secondary);max-width:400px;text-align:center">${esc(e?.toString() || 'Unknown error')}</div>
        <div style="display:flex;gap:12px;margin-top:16px">
          <button class="btn" id="back-to-meta-btn">Back</button>
          <button class="btn btn--primary" id="retry-pub-btn">Retry</button>
        </div>
      </div>
    `;

    content.querySelector('#back-to-meta-btn')?.addEventListener('click', () => {
      currentStep = 'metadata';
      renderPublish();
    });

    content.querySelector('#retry-pub-btn')?.addEventListener('click', () => {
      renderPublishingStep(content, displayName, tagline, category, version);
    });
  }
}

function renderSteps(active: PublishStep): string {
  const steps: { key: PublishStep; label: string }[] = [
    { key: 'select', label: '1. Select Skill' },
    { key: 'metadata', label: '2. Metadata' },
    { key: 'publishing', label: '3. Publish' },
  ];

  const activeIndex = steps.findIndex((s) => s.key === active);

  return `
    <div style="display:flex;gap:8px;margin-bottom:24px">
      ${steps
        .map(
          (step, i) => `
        <div style="
          display:flex;align-items:center;gap:6px;
          padding:8px 16px;border-radius:6px;font-size:13px;
          ${i === activeIndex ? 'background:var(--accent);color:#fff;font-weight:600' : i < activeIndex ? 'background:rgba(34,197,94,0.1);color:#22c55e' : 'background:var(--bg-secondary);color:var(--text-muted)'}
        ">
          ${i < activeIndex ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>' : ''}
          ${step.label}
        </div>
      `
        )
        .join('')}
    </div>
  `;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
