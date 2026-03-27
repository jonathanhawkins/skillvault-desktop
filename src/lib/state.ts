import type { LocalState, Package, ViewName } from './types';

type Listener = () => void;

interface AppState {
  currentView: ViewName;
  localState: LocalState | null;
  searchQuery: string;
  searchCategory: string | null;
  searchSort: string;
  searchResults: Package[];
  searchTotal: number;
  searchPage: number;
  trendingPackages: Package[];
  selectedPackage: Package | null;
  selectedAuthor: string;
  selectedName: string;
  selectedSkillName: string;
  selectedFilePath: string;
  selectedFileTitle: string;
  selectedPluginName: string;
  selectedPluginSource: string;
  loading: boolean;
  authenticated: boolean;
}

const state: AppState = {
  currentView: 'installed',
  localState: null,
  searchQuery: '',
  searchCategory: null,
  searchSort: 'trending',
  searchResults: [],
  searchTotal: 0,
  searchPage: 1,
  trendingPackages: [],
  selectedPackage: null,
  selectedAuthor: '',
  selectedName: '',
  selectedSkillName: '',
  selectedFilePath: '',
  selectedFileTitle: '',
  selectedPluginName: '',
  selectedPluginSource: 'claude',
  loading: false,
  authenticated: false,
};

const listeners: Listener[] = [];

export function getState(): Readonly<AppState> {
  return state;
}

export function setState(partial: Partial<AppState>) {
  Object.assign(state, partial);
  listeners.forEach((fn) => fn());
}

export function subscribe(fn: Listener): () => void {
  listeners.push(fn);
  return () => {
    const idx = listeners.indexOf(fn);
    if (idx >= 0) listeners.splice(idx, 1);
  };
}
