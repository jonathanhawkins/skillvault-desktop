import { getState, setState } from '../lib/state';
import { scanLocal, uninstallSkill, getMyPackages, deletePackage } from '../lib/api';
import { showToast } from '../components/toast';
import { renderSidebar } from '../components/sidebar';
import { navigate } from '../lib/router';
import { esc, formatBytes } from '../lib/utils';
import type { Package } from '../lib/types';

export async function renderInstalled() {
  const content = document.getElementById('content');
  if (!content) return;

  // Show loading state
  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">My Skills</h1>
      </div>
      <button class="btn btn--sm" id="scan-btn">Scan</button>
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  // Scan if no local state
  let state = getState();
  if (!state.localState) {
    try {
      const localState = await scanLocal();
      setState({ localState });
      renderSidebar();
    } catch (e: any) {
      content.innerHTML = `
        <div class="view-header">
          <div class="view-header-title">
            <h1 class="h1">My Skills</h1>
          </div>
        </div>
        <div class="empty-state">
          <div class="empty-state-text">Failed to scan: ${esc(e?.toString() || 'Unknown error')}</div>
          <button class="btn btn--sm" id="retry-btn">Retry</button>
        </div>
      `;
      content.querySelector('#retry-btn')?.addEventListener('click', () => renderInstalled());
      return;
    }
  }

  state = getState();
  const ls = state.localState!;

  // Group skills: global vs project-scoped
  const globalSkills = ls.skills.filter(s => !s.project);
  const projectGroups = new Map<string, typeof ls.skills>();
  for (const s of ls.skills) {
    if (s.project) {
      if (!projectGroups.has(s.project)) projectGroups.set(s.project, []);
      projectGroups.get(s.project)!.push(s);
    }
  }

  const renderSkillCard = (skill: typeof ls.skills[0]) => `
    <div class="skill-card skill-card--clickable" data-skill="${esc(skill.name)}" data-skill-path="${esc(skill.path)}">
      <div class="skill-card-header">
        <div class="skill-card-name">${esc(skill.name)}</div>
        <div style="display:flex;align-items:center;gap:6px">
          <span class="skill-card-source skill-card-source--${skill.source}">${skill.source}</span>
          ${skill.source === 'skillvault' ? `<button class="skill-card-delete" data-delete-skill="${esc(skill.name)}" title="Uninstall" aria-label="Uninstall ${esc(skill.name)}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
          </button>` : ''}
        </div>
      </div>
      ${skill.description ? `<div class="skill-card-desc">${esc(skill.description)}</div>` : ''}
      <div class="skill-card-meta">
        <span>${skill.file_count} files</span>
        ${skill.has_scripts ? '<span>scripts</span>' : ''}
        ${skill.has_subagents ? '<span>subagents</span>' : ''}
        ${skill.has_references ? '<span>references</span>' : ''}
        ${skill.installed_version ? `<span>v${esc(skill.installed_version)}</span>` : ''}
      </div>
    </div>`;

  const globalSkillsHtml = globalSkills.length > 0
    ? `<div class="grid">${globalSkills.map(renderSkillCard).join('')}</div>`
    : `<div class="empty-state">
        <svg class="empty-state-icon" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
        <div class="empty-state-text">No global skills installed yet.</div>
        <button class="btn btn--primary btn--sm" id="browse-btn">Browse Marketplace</button>
      </div>`;

  const projectSkillsHtml = Array.from(projectGroups.entries()).map(([project, skills]) => `
    <div class="installed-section" style="margin-top:24px">
      <div class="installed-section-header">
        <span class="installed-section-label">${esc(project)} Skills</span>
        <span class="installed-section-count">${skills.length}</span>
      </div>
      <div class="grid">${skills.map(renderSkillCard).join('')}</div>
    </div>
  `).join('');

  const agentsHtml = ls.agents.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Agents</span>
          <span class="installed-section-count">${ls.agents.length}</span>
        </div>
        <div class="grid">${ls.agents.map(agent => `
          <div class="skill-card skill-card--clickable" data-agent-path="${esc(agent.path)}" data-agent-name="${esc(agent.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(agent.name)}</div>
              <span class="skill-card-source skill-card-source--local">agent</span>
            </div>
            ${agent.description ? `<div class="skill-card-desc">${esc(agent.description)}</div>` : ''}
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const hooksHtml = ls.hooks.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Hooks</span>
          <span class="installed-section-count">${ls.hooks.length}</span>
        </div>
        <div class="grid">${ls.hooks.map(hook => `
          <div class="skill-card">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(hook.event)}</div>
              <span class="skill-card-source skill-card-source--local">hook</span>
            </div>
            <div class="skill-card-desc" style="font-family:'Geist Mono',monospace;font-size:11px">${esc(hook.command)}</div>
            ${hook.matcher ? `<div class="skill-card-meta"><span>matcher: ${esc(hook.matcher)}</span></div>` : ''}
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const pluginsHtml = ls.plugins.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Plugins</span>
          <span class="installed-section-count">${ls.plugins.length}</span>
        </div>
        <div class="grid">${ls.plugins.map(plugin => `
          <div class="skill-card skill-card--clickable" data-plugin-name="${esc(plugin.name)}" data-plugin-marketplace="${esc(plugin.marketplace)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(plugin.name)}</div>
              <span class="skill-card-source skill-card-source--local">v${esc(plugin.version)}</span>
            </div>
            <div class="skill-card-meta">
              <span>${esc(plugin.marketplace)}</span>
              <span>${esc(plugin.scope)}</span>
            </div>
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const mcpServersHtml = ls.mcp_servers.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">MCP Servers</span>
          <span class="installed-section-count">${ls.mcp_servers.length}</span>
        </div>
        <div class="grid">${ls.mcp_servers.map(server => `
          <div class="skill-card">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(server.name)}</div>
              <span class="skill-card-source skill-card-source--local">${esc(server.server_type)}</span>
            </div>
            <div class="skill-card-desc" style="font-family:'Geist Mono',monospace;font-size:11px">${server.url ? esc(server.url) : server.command ? esc(server.command) : ''}</div>
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const teamsHtml = ls.teams.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Teams</span>
          <span class="installed-section-count">${ls.teams.length}</span>
        </div>
        <div class="grid">${ls.teams.map(team => `
          <div class="skill-card skill-card--clickable" data-team-path="${esc(team.path)}" data-team-name="${esc(team.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(team.name)}</div>
              <span class="skill-card-source skill-card-source--local">team</span>
            </div>
            ${team.description ? `<div class="skill-card-desc">${esc(team.description)}</div>` : ''}
            <div class="skill-card-meta">
              <span>${team.member_count} member${team.member_count !== 1 ? 's' : ''}</span>
            </div>
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const rulesHtml = ls.rules.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Rules (CLAUDE.md)</span>
          <span class="installed-section-count">${ls.rules.length}</span>
        </div>
        <div class="grid">${ls.rules.map(rule => `
          <div class="skill-card skill-card--clickable" data-rule-path="${esc(rule.path)}" data-rule-name="${esc(rule.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(rule.name)}</div>
              <span class="skill-card-source skill-card-source--local">${formatBytes(rule.size_bytes)}</span>
            </div>
            <div class="skill-card-desc" style="font-family:'Geist Mono',monospace;font-size:11px;white-space:pre-wrap">${esc(rule.preview)}${rule.size_bytes > 200 ? '...' : ''}</div>
            ${rule.project_path ? `<div class="skill-card-meta"><span>${esc(rule.project_path)}</span></div>` : ''}
          </div>
        `).join('')}</div>
      </div>`
    : '';

  // Fetch published packages if authenticated
  let publishedHtml = '';
  let publishedPackages: Package[] = [];
  if (state.authenticated) {
    try {
      publishedPackages = await getMyPackages();
      if (publishedPackages.length > 0) {
        publishedHtml = `
          <div class="installed-section" style="margin-bottom:24px">
            <div class="installed-section-header">
              <span class="installed-section-label">Published by You</span>
              <span class="installed-section-count">${publishedPackages.length}</span>
            </div>
            <div class="grid">${publishedPackages.map(pkg => `
              <div class="skill-card skill-card--clickable" data-pub-author="${esc(pkg.author_id)}" data-pub-name="${esc(pkg.name)}">
                <div class="skill-card-header">
                  <div class="skill-card-name">${esc(pkg.display_name || pkg.name)}</div>
                  <div style="display:flex;align-items:center;gap:6px">
                    <span class="skill-card-source skill-card-source--skillvault">v${esc(pkg.current_version)}</span>
                    <button class="skill-card-delete" data-unpub-author="${esc(pkg.author_id)}" data-unpub-name="${esc(pkg.name)}" data-unpub-display="${esc(pkg.display_name || pkg.name)}" title="Unpublish" aria-label="Unpublish ${esc(pkg.name)}">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
                    </button>
                  </div>
                </div>
                ${pkg.tagline ? `<div class="skill-card-desc">${esc(pkg.tagline)}</div>` : ''}
                <div class="skill-card-meta">
                  <span>${esc(pkg.category)}</span>
                  <span>${pkg.download_count} downloads</span>
                </div>
              </div>
            `).join('')}</div>
          </div>`;
      }
    } catch {
      // Silently skip if fetch fails
    }
  }

  const hasCodexContent = ls.codex_config || ls.codex_rules.length > 0 || ls.codex_skills.length > 0 || ls.codex_agents.length > 0;

  const codexSeparatorHtml = hasCodexContent
    ? `<div style="margin:32px 0 24px;padding-top:24px;border-top:2px solid var(--border)">
        <div class="installed-section-header">
          <span class="installed-section-label" style="font-size:13px">Codex (OpenAI)</span>
        </div>
      </div>`
    : '';

  const codexConfigHtml = ls.codex_config
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Codex Config</span>
        </div>
        <div class="grid">
          <div class="skill-card skill-card--clickable" data-codex-config-path="${esc(ls.codex_config.config_path)}">
            <div class="skill-card-header">
              <div class="skill-card-name">Configuration</div>
              <span class="skill-card-source skill-card-source--local">codex</span>
            </div>
            ${ls.codex_config.model ? `<div style="font-size:13px;color:var(--text-secondary);margin:4px 0">Model: <strong style="color:var(--text-primary)">${esc(ls.codex_config.model)}</strong></div>` : ''}
            ${ls.codex_config.trusted_projects.length > 0 ? `<div style="margin:8px 0">
              <div style="font-family:'Geist Mono',monospace;font-size:10px;color:var(--text-faint);margin-bottom:4px;letter-spacing:0.5px">TRUSTED PROJECTS</div>
              ${ls.codex_config.trusted_projects.map(p => {
                const short = p.replace(/^\/Users\/[^/]+\//, '~/');
                return `<div style="font-family:'Geist Mono',monospace;font-size:11px;color:var(--text-muted);padding:2px 0">${esc(short)}</div>`;
              }).join('')}
            </div>` : ''}
            <div style="font-family:'Geist Mono',monospace;font-size:10px;color:var(--text-faint);margin-top:6px;padding-top:6px;border-top:1px solid var(--border)">${esc(ls.codex_config.config_path.replace(/^\/Users\/[^/]+\//, '~/'))}</div>
          </div>
        </div>
      </div>`
    : '';

  const codexRulesHtml = ls.codex_rules.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Codex Rules</span>
          <span class="installed-section-count">${ls.codex_rules.length}</span>
        </div>
        <div class="grid">${ls.codex_rules.map(rule => `
          <div class="skill-card skill-card--clickable" data-codex-rule-path="${esc(rule.path)}" data-codex-rule-name="${esc(rule.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(rule.name)}</div>
              <span class="skill-card-source skill-card-source--local">${rule.project ? esc(rule.project) : 'global'}</span>
            </div>
            <div class="skill-card-desc" style="font-family:'Geist Mono',monospace;font-size:11px;white-space:pre-wrap">${esc(rule.preview)}</div>
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const codexSkillsHtml = ls.codex_skills.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Codex Skills</span>
          <span class="installed-section-count">${ls.codex_skills.length}</span>
        </div>
        <div class="grid">${ls.codex_skills.map(skill => `
          <div class="skill-card skill-card--clickable" data-codex-skill-path="${esc(skill.path)}" data-codex-skill-name="${esc(skill.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(skill.name)}</div>
              <span class="skill-card-source skill-card-source--local">${skill.project ? esc(skill.project) : 'global'}</span>
            </div>
            ${skill.description ? `<div class="skill-card-desc">${esc(skill.description)}</div>` : ''}
          </div>
        `).join('')}</div>
      </div>`
    : '';

  const codexAgentsHtml = ls.codex_agents.length > 0
    ? `<div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Codex Agents</span>
          <span class="installed-section-count">${ls.codex_agents.length}</span>
        </div>
        <div class="grid">${ls.codex_agents.map(agent => `
          <div class="skill-card skill-card--clickable" data-codex-agent-path="${esc(agent.path)}" data-codex-agent-name="${esc(agent.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(agent.name)}</div>
              <span class="skill-card-source skill-card-source--local">${agent.project ? esc(agent.project) : 'global'}</span>
            </div>
          </div>
        `).join('')}</div>
      </div>`
    : '';

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">My Skills</h1>
      </div>
      <div style="display:flex;gap:8px;align-items:center">
        <select class="search-select" id="platform-filter" style="height:36px;font-size:10px">
          <option value="all"${state.installedPlatformFilter === 'all' ? ' selected' : ''}>All Platforms</option>
          <option value="claude"${state.installedPlatformFilter === 'claude' ? ' selected' : ''}>Claude Code</option>
          <option value="codex"${state.installedPlatformFilter === 'codex' ? ' selected' : ''}>Codex</option>
        </select>
        <button class="btn btn--sm" id="scan-btn">Scan</button>
      </div>
    </div>
    <div id="claude-sections">
      <div style="margin-bottom:16px;font-family:'Geist Mono',monospace;font-size:11px;color:var(--text-faint);letter-spacing:0.5px">CLAUDE CODE</div>
      ${publishedHtml}
      <div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">Global Skills</span>
          <span class="installed-section-count">${globalSkills.length}</span>
        </div>
        ${globalSkillsHtml}
      </div>
      ${projectSkillsHtml}
      ${agentsHtml}
      ${hooksHtml}
      ${pluginsHtml}
      ${mcpServersHtml}
      ${teamsHtml}
      ${rulesHtml}
    </div>
    <div id="codex-sections">
      ${codexSeparatorHtml}
      ${codexConfigHtml}
      ${codexRulesHtml}
      ${codexSkillsHtml}
      ${codexAgentsHtml}
    </div>
  `;

  // Bind events
  content.querySelector('#scan-btn')?.addEventListener('click', async () => {
    setState({ localState: null });
    renderInstalled();
  });

  content.querySelector('#browse-btn')?.addEventListener('click', () => {
    navigate('browse');
  });

  // Platform filter — persisted in state
  const platformFilter = content.querySelector('#platform-filter') as HTMLSelectElement;
  const applyPlatformFilter = (val: string) => {
    const claudeSections = content.querySelector('#claude-sections') as HTMLElement;
    const codexSections = content.querySelector('#codex-sections') as HTMLElement;
    if (claudeSections) claudeSections.style.display = (val === 'codex') ? 'none' : '';
    if (codexSections) codexSections.style.display = (val === 'claude') ? 'none' : '';
  };
  // Apply saved filter on load
  applyPlatformFilter(state.installedPlatformFilter);
  platformFilter?.addEventListener('change', () => {
    setState({ installedPlatformFilter: platformFilter.value });
    applyPlatformFilter(platformFilter.value);
  });

  // Click skill card to navigate to detail
  content.querySelectorAll('.skill-card[data-skill]').forEach((card) => {
    card.addEventListener('click', (e) => {
      // Don't navigate if clicking delete button
      if ((e.target as HTMLElement).closest('.skill-card-delete')) return;
      const el = card as HTMLElement;
      const skillPath = el.dataset.skillPath || '';
      setState({ selectedSkillName: el.dataset.skill || '', selectedFilePath: skillPath });
      navigate('skill-detail');
    });

    // Right-click — just navigate to detail
    card.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      const el = card as HTMLElement;
      const skillPath = el.dataset.skillPath || '';
      setState({ selectedSkillName: el.dataset.skill || '', selectedFilePath: skillPath });
      navigate('skill-detail');
    });
  });

  // Trash icon delete buttons (only on SkillVault-sourced skills)
  content.querySelectorAll('.skill-card-delete').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const name = (btn as HTMLElement).dataset.deleteSkill;
      if (!name) return;

      // Replace the button with confirm/cancel inline
      const parent = btn.parentElement;
      if (!parent) return;
      const original = parent.innerHTML;
      parent.innerHTML = `
        <span style="font-family:'Geist Mono',monospace;font-size:10px;color:var(--error)">Remove?</span>
        <button class="btn btn--sm btn--danger" data-confirm-delete="${esc(name)}" style="padding:3px 8px;font-size:9px">Yes</button>
        <button class="btn btn--sm" data-cancel-delete style="padding:3px 8px;font-size:9px">No</button>
      `;
      parent.querySelector('[data-confirm-delete]')?.addEventListener('click', (ev) => {
        ev.stopPropagation();
        uninstallSkill(name).then(() => {
          showToast(`Uninstalled "${name}"`, 'success');
          setState({ localState: null });
          renderInstalled();
        }).catch((err) => {
          showToast(`Failed: ${err}`, 'error');
        });
      });
      parent.querySelector('[data-cancel-delete]')?.addEventListener('click', (ev) => {
        ev.stopPropagation();
        parent.innerHTML = original;
      });
    });
  });

  // Click published package cards to view detail
  content.querySelectorAll('[data-pub-author]').forEach((card) => {
    card.addEventListener('click', (e) => {
      if ((e.target as HTMLElement).closest('.skill-card-delete')) return;
      const el = card as HTMLElement;
      const author = el.dataset.pubAuthor!;
      const name = el.dataset.pubName!;
      setState({ selectedAuthor: author, selectedName: name, selectedPackage: null });
      navigate('detail');
    });
  });

  // Unpublish buttons on published packages
  content.querySelectorAll('[data-unpub-name]').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const el = btn as HTMLElement;
      const author = el.dataset.unpubAuthor!;
      const name = el.dataset.unpubName!;
      const display = el.dataset.unpubDisplay || name;

      const parent = btn.parentElement;
      if (!parent) return;
      const original = parent.innerHTML;
      parent.innerHTML = `
        <span style="font-family:'Geist Mono',monospace;font-size:10px;color:var(--error)">Unpublish?</span>
        <button class="btn btn--sm btn--danger" data-confirm-unpub style="padding:3px 8px;font-size:9px">Yes</button>
        <button class="btn btn--sm" data-cancel-unpub style="padding:3px 8px;font-size:9px">No</button>
      `;
      parent.querySelector('[data-confirm-unpub]')?.addEventListener('click', (ev) => {
        ev.stopPropagation();
        deletePackage(author, name).then(() => {
          showToast(`Unpublished "${display}"`, 'success');
          renderInstalled();
        }).catch((err) => {
          showToast(`Failed: ${err}`, 'error');
        });
      });
      parent.querySelector('[data-cancel-unpub]')?.addEventListener('click', (ev) => {
        ev.stopPropagation();
        parent.innerHTML = original;
      });
    });
  });

  // Click agent cards to view content
  content.querySelectorAll('[data-agent-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.agentPath!;
      const name = (card as HTMLElement).dataset.agentName!;
      setState({ selectedFilePath: path, selectedFileTitle: name });
      navigate('file-detail');
    });
  });

  // Click plugin cards to view details
  content.querySelectorAll('[data-plugin-name]').forEach((card) => {
    card.addEventListener('click', () => {
      const name = (card as HTMLElement).dataset.pluginName!;
      setState({ selectedPluginName: name });
      navigate('plugin-detail');
    });
  });

  // Click team cards to view config
  content.querySelectorAll('[data-team-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.teamPath!;
      const name = (card as HTMLElement).dataset.teamName!;
      setState({ selectedFilePath: path + '/config.json', selectedFileTitle: name + ' (team)' });
      navigate('file-detail');
    });
  });

  // Click rule cards to view CLAUDE.md
  content.querySelectorAll('[data-rule-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.rulePath!;
      const name = (card as HTMLElement).dataset.ruleName!;
      setState({ selectedFilePath: path, selectedFileTitle: name + ' — CLAUDE.md' });
      navigate('file-detail');
    });
  });

  // Click Codex config card
  content.querySelectorAll('[data-codex-config-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.codexConfigPath!;
      setState({ selectedFilePath: path, selectedFileTitle: 'Codex Configuration' });
      navigate('file-detail');
    });
  });

  // Click Codex rule cards
  content.querySelectorAll('[data-codex-rule-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.codexRulePath!;
      const name = (card as HTMLElement).dataset.codexRuleName!;
      setState({ selectedFilePath: path, selectedFileTitle: name + ' — Codex Rule' });
      navigate('file-detail');
    });
  });

  // Click Codex skill cards
  content.querySelectorAll('[data-codex-skill-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.codexSkillPath!;
      const name = (card as HTMLElement).dataset.codexSkillName!;
      setState({ selectedFilePath: path, selectedFileTitle: name + ' — Codex Skill' });
      navigate('file-detail');
    });
  });

  // Click Codex agent cards
  content.querySelectorAll('[data-codex-agent-path]').forEach((card) => {
    card.addEventListener('click', () => {
      const path = (card as HTMLElement).dataset.codexAgentPath!;
      const name = (card as HTMLElement).dataset.codexAgentName!;
      setState({ selectedFilePath: path, selectedFileTitle: name + ' — Codex Agent' });
      navigate('file-detail');
    });
  });
}
