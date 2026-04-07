import { getState, setState } from '../lib/state';
import { scanLocal, packageSkill, packageSkills, publishSkill, publishSkills, getAuthStatus } from '../lib/api';
import { showToast } from '../components/toast';
import { renderSidebar } from '../components/sidebar';
import { navigate } from '../lib/router';
import { esc, formatBytes } from '../lib/utils';
import type { LocalSkill, PackagedSkill } from '../lib/types';

const CATEGORIES = [
  'automation',
  'coding',
  'configs',
  'data',
  'database',
  'deployment',
  'design',
  'devops',
  'docs',
  'gamedev',
  'ai',
  'learning',
  'mobile',
  'monitoring',
  'productivity',
  'security',
  'testing',
  'toolkit',
  'web',
  'other',
];

type PublishStep = 'select' | 'metadata' | 'publishing';

let currentStep: PublishStep = 'select';
let selectedSkills: LocalSkill[] = [];
let packaged: PackagedSkill | null = null;
let packageName = '';

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

function isSkillSelected(skill: LocalSkill): boolean {
  return selectedSkills.some((s) => s.name === skill.name && s.path === skill.path);
}

function toggleSkillSelection(skill: LocalSkill) {
  const idx = selectedSkills.findIndex((s) => s.name === skill.name && s.path === skill.path);
  if (idx >= 0) {
    selectedSkills.splice(idx, 1);
  } else {
    selectedSkills.push(skill);
  }
}

function groupSkillsByProject(skills: LocalSkill[]): Map<string, LocalSkill[]> {
  const groups = new Map<string, LocalSkill[]>();
  for (const skill of skills) {
    const key = skill.project ?? '__global__';
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(skill);
  }
  return groups;
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

  const grouped = groupSkillsByProject(localSkills);
  const selCount = selectedSkills.length;

  // Build grouped skill cards HTML
  let groupedCardsHtml = '';
  const sortedKeys = [...grouped.keys()].sort((a, b) => {
    if (a === '__global__') return -1;
    if (b === '__global__') return 1;
    return a.localeCompare(b);
  });

  for (const key of sortedKeys) {
    const skills = grouped.get(key)!;
    const headerLabel = key === '__global__' ? 'Global Skills' : esc(key);
    groupedCardsHtml += `<div class="publish-group-header">${headerLabel}</div>`;
    groupedCardsHtml += `<div class="grid">`;
    for (const skill of skills) {
      const selected = isSkillSelected(skill);
      groupedCardsHtml += `
        <div class="skill-card skill-card--clickable publish-select-card${selected ? ' publish-select-card--selected' : ''}" data-skill="${esc(skill.name)}" data-path="${esc(skill.path)}">
          <div style="display:flex;align-items:flex-start;gap:10px">
            <div class="publish-checkbox${selected ? ' publish-checkbox--checked' : ''}">
              ${selected ? '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>' : ''}
            </div>
            <div style="flex:1;min-width:0">
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
          </div>
        </div>
      `;
    }
    groupedCardsHtml += `</div>`;
  }

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <p style="font-size:13px;color:var(--text-secondary);margin-bottom:16px">
      Select skills to publish to skillvault.md
    </p>
    <div class="publish-select-toolbar">
      <span class="publish-select-count">${selCount} skill${selCount !== 1 ? 's' : ''} selected</span>
      <button class="btn btn--sm" id="select-all-btn">Select All</button>
      <button class="btn btn--sm" id="clear-btn"${selCount === 0 ? ' disabled' : ''}>Clear</button>
    </div>
    ${groupedCardsHtml}
    <div style="display:flex;justify-content:flex-end;margin-top:24px">
      <button class="btn btn--primary" id="next-btn"${selCount < 1 ? ' disabled' : ''}>Next</button>
    </div>
  `;

  // Bind card selection (toggle)
  content.querySelectorAll('.publish-select-card').forEach((card) => {
    card.addEventListener('click', () => {
      const name = (card as HTMLElement).dataset.skill;
      const path = (card as HTMLElement).dataset.path;
      if (!name || !path) return;
      const skill = localSkills.find((s) => s.name === name && s.path === path);
      if (!skill) return;

      toggleSkillSelection(skill);
      // Re-render to update all visual state
      renderSelectStep(content);
    });
  });

  // Select All
  content.querySelector('#select-all-btn')?.addEventListener('click', () => {
    selectedSkills = [...localSkills];
    renderSelectStep(content);
  });

  // Clear
  content.querySelector('#clear-btn')?.addEventListener('click', () => {
    selectedSkills = [];
    renderSelectStep(content);
  });

  // Next button
  content.querySelector('#next-btn')?.addEventListener('click', async () => {
    if (selectedSkills.length < 1) return;

    const nextBtn = content.querySelector('#next-btn') as HTMLButtonElement;
    if (nextBtn) {
      nextBtn.disabled = true;
      nextBtn.textContent = 'Packaging...';
    }

    try {
      if (selectedSkills.length === 1) {
        packaged = await packageSkill(selectedSkills[0].name);
      } else {
        packaged = await packageSkills(selectedSkills.map((s) => s.name), selectedSkills.map((s) => s.path));
      }
      // Generate default package name
      packageName = selectedSkills.length === 1
        ? selectedSkills[0].name
        : selectedSkills[0].name + '-bundle';
      currentStep = 'metadata';
      renderPublish();
    } catch (e: any) {
      showToast(`Failed to package skill${selectedSkills.length > 1 ? 's' : ''}: ${e}`, 'error');
      if (nextBtn) {
        nextBtn.disabled = false;
        nextBtn.textContent = 'Next';
      }
    }
  });
}

function renderMetadataStep(content: HTMLElement) {
  if (selectedSkills.length === 0 || !packaged) {
    currentStep = 'select';
    renderPublish();
    return;
  }

  const stepsHtml = renderSteps('metadata');
  const isBundle = selectedSkills.length > 1;

  // Derive default display name from package name (kebab-case to Title Case)
  const defaultDisplayName = packageName
    .split(/[-_]/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');

  const defaultTagline = packaged.description || '';
  const sizeStr = formatBytes(packaged.size_bytes);
  const totalFiles = selectedSkills.reduce((sum, s) => sum + s.file_count, 0);

  // Package name validation pattern
  const pkgNamePattern = 'a-z0-9-';

  // Chips for multi-select
  const chipsHtml = isBundle
    ? `
      <div class="publish-field">
        <label class="settings-label">Included Skills</label>
        <div style="display:flex;flex-wrap:wrap;gap:6px" id="skill-chips">
          ${selectedSkills
            .map(
              (s) =>
                `<span class="publish-chip" data-chip-name="${esc(s.name)}" data-chip-path="${esc(s.path)}">${esc(s.name)} <button type="button" aria-label="Remove ${esc(s.name)}">&#215;</button></span>`
            )
            .join('')}
        </div>
        <div style="font-size:12px;color:var(--text-faint);margin-top:8px">${selectedSkills.length} skills, ${totalFiles} files</div>
      </div>
    `
    : '';

  const summaryHtml = isBundle
    ? `
      <div style="margin-bottom:24px;padding:16px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:8px">
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:4px">
          <span style="font-weight:600;font-size:14px;color:var(--text-primary)">Bundle: ${esc(packageName)}</span>
          <span style="font-size:12px;color:var(--text-muted)">${selectedSkills.length} skills</span>
          <span style="font-size:12px;color:var(--text-muted)">${totalFiles} files</span>
          <span style="font-size:12px;color:var(--text-muted)">${sizeStr}</span>
        </div>
      </div>
    `
    : `
      <div style="margin-bottom:24px;padding:16px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:8px">
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:8px">
          <span style="font-weight:600;font-size:14px;color:var(--text-primary)">${esc(selectedSkills[0].name)}</span>
          <span style="font-size:12px;color:var(--text-muted)">${packaged.file_count} files</span>
          <span style="font-size:12px;color:var(--text-muted)">${sizeStr}</span>
        </div>
      </div>
    `;

  const packageNameFieldHtml = isBundle
    ? `
      <div class="publish-field">
        <label class="settings-label" for="pub-package-name">Package Name</label>
        <input class="settings-input" id="pub-package-name" type="text" value="${esc(packageName)}" placeholder="my-skill-bundle" pattern="[${pkgNamePattern}]+">
        <div class="settings-hint">Lowercase, alphanumeric and hyphens only</div>
      </div>
    `
    : '';

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <div style="max-width:560px">
      ${summaryHtml}
      ${packageNameFieldHtml}
      ${chipsHtml}

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
                `<option value="${cat}"${cat === 'coding' ? ' selected' : ''}>${cat.charAt(0).toUpperCase() + cat.slice(1)}</option>`
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

  // Chip removal handlers
  if (isBundle) {
    content.querySelectorAll('.publish-chip button').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const chip = (btn as HTMLElement).closest('.publish-chip') as HTMLElement;
        if (!chip) return;
        const chipName = chip.dataset.chipName;
        const chipPath = chip.dataset.chipPath;
        if (!chipName || !chipPath) return;
        selectedSkills = selectedSkills.filter(
          (s) => !(s.name === chipName && s.path === chipPath)
        );
        if (selectedSkills.length === 0) {
          currentStep = 'select';
          renderPublish();
        } else {
          // Re-render metadata with updated skills
          renderMetadataStep(content);
        }
      });
    });
  }

  // Package name live update
  const pkgNameInput = content.querySelector('#pub-package-name') as HTMLInputElement | null;
  if (pkgNameInput) {
    pkgNameInput.addEventListener('input', () => {
      packageName = pkgNameInput.value.trim().toLowerCase().replace(/[^a-z0-9-]/g, '');
      pkgNameInput.value = packageName;
    });
  }

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

    // Update packageName from input if bundle
    if (pkgNameInput) {
      packageName = pkgNameInput.value.trim().toLowerCase().replace(/[^a-z0-9-]/g, '');
    } else {
      packageName = selectedSkills[0].name;
    }

    if (!displayName) {
      showToast('Display name is required', 'error');
      return;
    }
    if (!version || !/^\d+\.\d+\.\d+/.test(version)) {
      showToast('Version must be in semver format (e.g. 1.0.0)', 'error');
      return;
    }
    if (isBundle && !packageName) {
      showToast('Package name is required', 'error');
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
  if (selectedSkills.length === 0) {
    currentStep = 'select';
    renderPublish();
    return;
  }

  const stepsHtml = renderSteps('publishing');
  const isBundle = selectedSkills.length > 1;
  const publishLabel = isBundle ? `Publishing ${selectedSkills.length} skills...` : 'Packaging...';

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Publish Skill</h1>
      </div>
    </div>
    ${stepsHtml}
    <div style="display:flex;flex-direction:column;align-items:center;padding:48px 0;gap:16px">
      <div class="spinner"></div>
      <div id="publish-status" style="font-size:14px;color:var(--text-secondary)">${publishLabel}</div>
    </div>
  `;

  if (!displayName || tagline == null || !category || !version) {
    currentStep = 'metadata';
    renderPublish();
    return;
  }

  const statusEl = content.querySelector('#publish-status');

  try {
    if (statusEl) statusEl.textContent = 'Uploading to skillvault.md...';

    let result: string;
    if (isBundle) {
      result = await publishSkills(
        selectedSkills.map((s) => s.name),
        selectedSkills.map((s) => s.path),
        packageName,
        displayName,
        tagline ?? '',
        category,
        version
      );
    } else {
      result = await publishSkill(
        selectedSkills[0].name,
        displayName,
        tagline ?? '',
        category,
        version
      );
    }

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
        // Extract "author/name" from result like "Published author/Display Name v1.0.0 to skillvault.md"
        const match = result.match(/Published\s+(\S+)\//);
        const author = match ? match[1] : '';
        const pkgName = isBundle ? packageName : selectedSkills[0]?.name || '';
        const url = author && pkgName
          ? `https://skillvault.md/${author}/${pkgName}`
          : 'https://skillvault.md/search';
        await open(url);
      } catch {
        // ignore
      }
    });

    content.querySelector('#done-btn')?.addEventListener('click', () => {
      currentStep = 'select';
      selectedSkills = [];
      packaged = null;
      packageName = '';
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
    { key: 'select', label: '1. Select Skills' },
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
