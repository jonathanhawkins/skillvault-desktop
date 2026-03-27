import { getState } from '../lib/state';
import { navigate } from '../lib/router';

const LOGO_SVG = `<svg width="36" height="36" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="5" width="22" height="22" rx="2.5" stroke="#eeeeee" stroke-width="2" fill="none"/><circle cx="14" cy="16" r="7" stroke="#eeeeee" stroke-width="2" fill="none"/><line x1="14" y1="10.5" x2="14" y2="8" stroke="#ee6018" stroke-width="2.2" stroke-linecap="round"/><line x1="14" y1="10.5" x2="18.2" y2="12.8" stroke="#ee6018" stroke-width="2.2" stroke-linecap="round"/><line x1="14" y1="10.5" x2="9.8" y2="12.8" stroke="#ee6018" stroke-width="2.2" stroke-linecap="round"/><circle cx="14" cy="10.5" r="1.5" fill="#ee6018"/><rect x="26" y="9" width="2.5" height="2.5" rx="0.5" fill="#eeeeee"/><rect x="26" y="14" width="2.5" height="2.5" rx="0.5" fill="#eeeeee"/><rect x="26" y="19" width="2.5" height="2.5" rx="0.5" fill="#eeeeee"/></svg>`;

const ICONS = {
  installed: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>`,
  browse: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`,
  recent: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>`,
  trending: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/><polyline points="17 6 23 6 23 12"/></svg>`,
  publish: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>`,
  plugins: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 7V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v3"/></svg>`,
  settings: `<svg class="sidebar-link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>`,
};

export function renderSidebar() {
  const el = document.getElementById('sidebar');
  if (!el) return;

  const state = getState();
  const skillCount = state.localState?.skills.length ?? 0;

  el.innerHTML = `
    <div class="sidebar">
      <div class="sidebar-header">
        <div class="sidebar-logo">
          ${LOGO_SVG}
          <span>Skill Vault</span>
        </div>
        <span class="sidebar-badge">Desktop</span>
      </div>
      <nav class="sidebar-nav">
        <div class="sidebar-section">
          <div class="sidebar-section-label">Library</div>
        </div>
        <a class="sidebar-link${state.currentView === 'installed' ? ' sidebar-link--active' : ''}" data-view="installed">
          ${ICONS.installed}
          <span>My Skills</span>
          ${skillCount > 0 ? `<span class="sidebar-link-badge">${skillCount}</span>` : ''}
        </a>
        <a class="sidebar-link${state.currentView === 'publish' ? ' sidebar-link--active' : ''}" data-view="publish">
          ${ICONS.publish}
          <span>Publish</span>
        </a>
        <div class="sidebar-section">
          <div class="sidebar-section-label">Marketplace</div>
        </div>
        <a class="sidebar-link${state.currentView === 'browse' ? ' sidebar-link--active' : ''}" data-view="browse">
          ${ICONS.browse}
          <span>Browse</span>
        </a>
        <a class="sidebar-link${state.currentView === 'recent' ? ' sidebar-link--active' : ''}" data-view="recent">
          ${ICONS.recent}
          <span>New</span>
        </a>
        <a class="sidebar-link${state.currentView === 'trending' ? ' sidebar-link--active' : ''}" data-view="trending">
          ${ICONS.trending}
          <span>Trending</span>
        </a>
        <a class="sidebar-link${state.currentView === 'plugins' || state.currentView === 'plugin-detail' ? ' sidebar-link--active' : ''}" data-view="plugins">
          ${ICONS.plugins}
          <span>Plugins</span>
        </a>
        <div class="sidebar-divider"></div>
        <a class="sidebar-link${state.currentView === 'settings' ? ' sidebar-link--active' : ''}" data-view="settings">
          ${ICONS.settings}
          <span>Settings</span>
        </a>
      </nav>
      <div class="sidebar-footer">
        <div class="sidebar-user" data-view="settings">
          ${state.authenticated
            ? '<span style="display:flex;align-items:center;gap:6px"><svg width="8" height="8" viewBox="0 0 8 8"><circle cx="4" cy="4" r="4" fill="#22c55e"/></svg> Connected</span>'
            : 'Connect Account'}
        </div>
      </div>
    </div>
  `;

  // Bind navigation
  el.querySelectorAll('[data-view]').forEach((link) => {
    link.addEventListener('click', async (e) => {
      e.preventDefault();
      const el = link as HTMLElement;
      const view = el.getAttribute('data-view') as any;

      if (view) navigate(view);
    });
  });
}
