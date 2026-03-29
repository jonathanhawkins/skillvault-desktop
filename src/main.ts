import { renderSidebar } from './components/sidebar';
import { registerView, renderCurrentView } from './lib/router';
import { getAuthStatus } from './lib/api';
import { getState, setState } from './lib/state';

// Import views
import { renderInstalled } from './views/installed';
import { renderBrowse } from './views/browse';
import { renderTrending } from './views/trending';
import { renderRecent } from './views/recent';
import { renderDetail } from './views/detail';
import { renderSettings } from './views/settings';
import { renderSkillDetail } from './views/skill-detail';
import { renderFileDetail } from './views/file-detail';
import { renderPlugins } from './views/plugins';
import { renderPluginDetail } from './views/plugin-detail';
import { renderPublish } from './views/publish';
import { renderEditPackage } from './views/edit-package';

// Register all views
registerView('installed', renderInstalled);
registerView('browse', renderBrowse);
registerView('recent', renderRecent);
registerView('trending', renderTrending);
registerView('detail', renderDetail);
registerView('settings', renderSettings);
registerView('skill-detail', renderSkillDetail);
registerView('file-detail', renderFileDetail);
registerView('plugins', renderPlugins);
registerView('plugin-detail', renderPluginDetail);
registerView('publish', renderPublish);
registerView('edit-package', renderEditPackage);

// Initialize app
async function init() {
  // Check auth status
  try {
    const auth = await getAuthStatus();
    setState({ authenticated: auth.authenticated, username: auth.username ?? null });
  } catch {
    // Not authenticated
  }

  // Render sidebar
  renderSidebar();

  // Render default view
  renderCurrentView();

  // Listen for file system changes (from Rust file watcher)
  const { listen } = window.__TAURI__.event;
  listen('local-state-changed', () => {
    // Clear cached local state to force re-scan on next view
    setState({ localState: null });
    // If currently on installed view, refresh it
    if (getState().currentView === 'installed') {
      renderCurrentView();
    }
  });

  // Listen for available updates (from background update checker)
  listen<number>('updates-available', (event) => {
    const count = event.payload;
    const badge = document.querySelector('[data-view="installed"] .sidebar-link-badge');
    if (badge) {
      // Append update indicator to existing badge
      badge.setAttribute('title', `${count} update(s) available`);
      badge.classList.add('has-updates');
    } else {
      // Add a new update badge to the My Skills link
      const link = document.querySelector('[data-view="installed"]');
      if (link) {
        const span = document.createElement('span');
        span.className = 'sidebar-link-badge has-updates';
        span.textContent = `${count}`;
        span.title = `${count} update(s) available`;
        link.appendChild(span);
      }
    }
  });
}

// Wait for DOM
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
