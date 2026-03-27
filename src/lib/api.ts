import type {
  LocalState,
  PackageSearchResult,
  Package,
  PackagedSkill,
  CategoryCount,
  PlatformStats,
  UpdateInfo,
  AuthStatus,
  SkillDetail,
  MarketplacePlugin,
  PluginDetail,
} from './types';

const { invoke } = window.__TAURI__.core;

export async function scanLocal(): Promise<LocalState> {
  return invoke('scan_local');
}

export async function searchPackages(
  query: string,
  category: string | null,
  sort: string | null,
  page: number,
  limit: number,
  compat?: string | null
): Promise<PackageSearchResult> {
  return invoke('search_packages', { query, category, sort, page, limit, compat: compat ?? null });
}

export async function getPackage(author: string, name: string): Promise<Package> {
  return invoke('get_package', { author, name });
}

export async function getTrending(): Promise<Package[]> {
  return invoke('get_trending');
}

export async function getCategories(): Promise<CategoryCount[]> {
  return invoke('get_categories');
}

export async function getPlatformStats(): Promise<PlatformStats> {
  return invoke('get_platform_stats');
}

export async function installPackage(author: string, name: string, installPath?: string | null): Promise<string> {
  return invoke('install_package', { author, name, installPath: installPath ?? null });
}

export async function listProjects(): Promise<Array<{ name: string; path: string; encoded_name: string }>> {
  return invoke('list_projects');
}

export async function uninstallSkill(skillName: string): Promise<void> {
  return invoke('uninstall_skill', { skillName });
}

export async function checkUpdates(): Promise<UpdateInfo[]> {
  return invoke('check_updates');
}

export async function setAuthToken(token: string): Promise<void> {
  return invoke('set_auth_token', { token });
}

export async function getAuthStatus(): Promise<AuthStatus> {
  return invoke('get_auth_status');
}

export async function clearAuthToken(): Promise<void> {
  return invoke('clear_auth_token');
}

export async function getSkillDetail(skillName: string, skillPath?: string): Promise<SkillDetail> {
  return invoke('get_skill_detail', { skillName, skillPath: skillPath || null });
}

export async function readFileContent(filePath: string): Promise<string> {
  return invoke('read_file_content', { filePath });
}

export async function getMarketplacePlugins(): Promise<MarketplacePlugin[]> {
  return invoke('get_marketplace_plugins');
}

export async function getPluginDetail(pluginName: string, pluginSource?: string): Promise<PluginDetail> {
  return invoke('get_plugin_detail', { pluginName, pluginSource: pluginSource ?? null });
}

export async function packageSkill(skillName: string): Promise<PackagedSkill> {
  return invoke('package_skill', { skillName });
}

export async function publishSkill(
  skillName: string,
  displayName: string,
  tagline: string,
  category: string,
  version: string
): Promise<string> {
  return invoke('publish_skill', { skillName, displayName, tagline, category, version });
}
