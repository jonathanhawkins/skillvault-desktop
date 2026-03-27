import type { ViewName } from './types';
import { getState, setState } from './state';

const viewRenderers: Record<ViewName, () => void> = {} as any;

export function registerView(name: ViewName, render: () => void) {
  viewRenderers[name] = render;
}

export function navigate(view: ViewName, params?: Record<string, string>) {
  setState({ currentView: view, ...params } as any);
  renderCurrentView();
  updateSidebarActive();
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
