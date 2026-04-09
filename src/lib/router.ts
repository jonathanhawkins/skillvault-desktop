import type { ViewName } from './types';
import { getState, setState } from './state';

const viewRenderers: Record<ViewName, () => void> = {} as any;
const history: ViewName[] = [];
let historyIndex = -1;
let navigatingHistory = false;

export function registerView(name: ViewName, render: () => void) {
  viewRenderers[name] = render;
}

export function navigate(view: ViewName, params?: Record<string, string>) {
  // Don't push to history if we're navigating via back/forward
  if (!navigatingHistory) {
    // Trim any forward history
    if (historyIndex < history.length - 1) {
      history.splice(historyIndex + 1);
    }
    history.push(view);
    historyIndex = history.length - 1;

    const MAX_HISTORY = 50;
    if (history.length > MAX_HISTORY) {
      history.splice(0, history.length - MAX_HISTORY);
      historyIndex = history.length - 1;
    }
  }

  setState({ currentView: view, ...params } as any);
  renderCurrentView();
  updateSidebarActive();
}

export function goBack() {
  if (historyIndex > 0) {
    historyIndex--;
    navigatingHistory = true;
    navigate(history[historyIndex]);
    navigatingHistory = false;
  }
}

export function goForward() {
  if (historyIndex < history.length - 1) {
    historyIndex++;
    navigatingHistory = true;
    navigate(history[historyIndex]);
    navigatingHistory = false;
  }
}

export function canGoBack(): boolean {
  return historyIndex > 0;
}

export function renderCurrentView() {
  const { currentView } = getState();
  const render = viewRenderers[currentView];
  if (render) render();
}

function updateSidebarActive() {
  const { currentView } = getState();
  document.querySelectorAll('.sidebar-link').forEach((el) => {
    const view = el.getAttribute('data-view');
    el.classList.toggle('sidebar-link--active', view === currentView);
  });
}

// Keyboard shortcuts for navigation
document.addEventListener('keydown', (e) => {
  // Don't trigger shortcuts when typing in inputs
  const tag = (e.target as HTMLElement).tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

  // Cmd+[ or Alt+Cmd+Left = Back
  if (e.metaKey && (e.key === '[' || (e.key === 'ArrowLeft' && e.altKey))) {
    e.preventDefault();
    goBack();
  }
  // Cmd+] or Alt+Cmd+Right = Forward
  if (e.metaKey && (e.key === ']' || (e.key === 'ArrowRight' && e.altKey))) {
    e.preventDefault();
    goForward();
  }

  // View shortcuts (numbers 1-9)
  if (e.metaKey && !e.shiftKey && !e.altKey) {
    switch (e.key) {
      case '1': e.preventDefault(); navigate('installed'); break;
      case '2': e.preventDefault(); navigate('publish'); break;
      case '3': e.preventDefault(); navigate('browse'); break;
      case '4': e.preventDefault(); navigate('recent'); break;
      case '5': e.preventDefault(); navigate('trending'); break;
      case '6': e.preventDefault(); navigate('plugins'); break;
      case '7': e.preventDefault(); navigate('settings'); break;
      case '8': e.preventDefault(); navigate('optimize'); break;
    }
  }

  // Cmd+, = Settings (standard macOS)
  if (e.metaKey && e.key === ',') {
    e.preventDefault();
    navigate('settings');
  }

  // Cmd+F = Focus search (if on Browse or Plugins view)
  if (e.metaKey && e.key === 'f') {
    const searchInput = document.querySelector('#search-input, #plugin-search') as HTMLInputElement;
    if (searchInput) {
      e.preventDefault();
      searchInput.focus();
      searchInput.select();
    }
  }

  // Cmd+R = Refresh/Scan
  if (e.metaKey && e.key === 'r') {
    e.preventDefault();
    const scanBtn = document.querySelector('#scan-btn') as HTMLButtonElement;
    if (scanBtn) {
      scanBtn.click();
    } else {
      // Re-render current view
      renderCurrentView();
    }
  }

  // Escape = Go back (when not in an input)
  if (e.key === 'Escape') {
    goBack();
  }
});

// Mouse back/forward buttons (mouse button 3 = back, 4 = forward)
document.addEventListener('mouseup', (e) => {
  if (e.button === 3) {
    e.preventDefault();
    goBack();
  }
  if (e.button === 4) {
    e.preventDefault();
    goForward();
  }
});
