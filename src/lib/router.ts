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
  // Cmd+[ or Cmd+Left = Back
  if (e.metaKey && (e.key === '[' || (e.key === 'ArrowLeft' && e.altKey))) {
    e.preventDefault();
    goBack();
  }
  // Cmd+] or Cmd+Right = Forward
  if (e.metaKey && (e.key === ']' || (e.key === 'ArrowRight' && e.altKey))) {
    e.preventDefault();
    goForward();
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
