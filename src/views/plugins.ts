import { getState, setState } from '../lib/state';
import { getMarketplacePlugins } from '../lib/api';
import { navigate } from '../lib/router';
import type { MarketplacePlugin } from '../lib/types';

export async function renderPlugins() {
  const content = document.getElementById('content');
  if (!content) return;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Plugins</h1>
      </div>
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  let plugins: MarketplacePlugin[];
  try {
    plugins = await getMarketplacePlugins();
  } catch (e: any) {
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Plugins</h1>
        </div>
      </div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load plugins: ${esc(e?.toString() || 'Unknown error')}</div>
        <button class="btn btn--sm" id="retry-btn">Retry</button>
      </div>
    `;
    content.querySelector('#retry-btn')?.addEventListener('click', () => renderPlugins());
    return;
  }

  // Extract unique categories
  const categories = Array.from(new Set(plugins.map(p => p.category).filter(Boolean) as string[])).sort();

  const s = getState();
  renderPluginList(content, plugins, categories, '', s.pluginCategoryFilter, s.pluginSourceFilter);
}

function renderPluginList(
  content: HTMLElement,
  allPlugins: MarketplacePlugin[],
  categories: string[],
  searchQuery: string,
  selectedCategory: string,
  selectedSource: string
) {
  // Filter plugins
  let filtered = allPlugins;
  if (searchQuery) {
    const q = searchQuery.toLowerCase();
    filtered = filtered.filter(p =>
      p.name.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.keywords.some(k => k.toLowerCase().includes(q))
    );
  }
  if (selectedCategory !== 'all') {
    filtered = filtered.filter(p => p.category === selectedCategory);
  }
  if (selectedSource !== 'all') {
    filtered = filtered.filter(p => p.source === selectedSource);
  }

  // Group by category
  const grouped: Record<string, MarketplacePlugin[]> = {};
  for (const p of filtered) {
    const cat = p.category || 'uncategorized';
    if (!grouped[cat]) grouped[cat] = [];
    grouped[cat].push(p);
  }
  const sortedCategories = Object.keys(grouped).sort();

  const categoryOptions = categories.map(c =>
    `<option value="${esc(c)}"${selectedCategory === c ? ' selected' : ''}>${esc(c)}</option>`
  ).join('');

  const groupsHtml = sortedCategories.length > 0
    ? sortedCategories.map(cat => `
      <div class="installed-section">
        <div class="installed-section-header">
          <span class="installed-section-label">${esc(cat)}</span>
          <span class="installed-section-count">${grouped[cat].length}</span>
        </div>
        <div class="grid">${grouped[cat].map(plugin => `
          <div class="skill-card skill-card--clickable" data-plugin-browse="${esc(plugin.name)}" data-plugin-source="${esc(plugin.source)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(plugin.name)}</div>
              ${plugin.is_installed
                ? '<span class="skill-card-source skill-card-source--skillvault">INSTALLED</span>'
                : '<span class="skill-card-source" style="color:var(--text-faint);border-color:var(--border)">AVAILABLE</span>'}
            </div>
            <div class="skill-card-desc">${esc(plugin.description)}</div>
            <div class="skill-card-meta">
              <span style="color:${plugin.source === 'codex' ? '#10b981' : 'var(--accent)'}">${plugin.source === 'codex' ? 'Codex' : 'Claude Code'}</span>
              ${plugin.category ? `<span>${esc(plugin.category)}</span>` : ''}
              ${plugin.author_name ? `<span>${esc(plugin.author_name)}</span>` : ''}
            </div>
          </div>
        `).join('')}</div>
      </div>
    `).join('')
    : `<div class="empty-state">
        <div class="empty-state-text">No plugins found.</div>
      </div>`;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Plugins</h1>
      </div>
    </div>
    <div style="display:flex;gap:8px;margin-bottom:24px;flex-wrap:wrap">
      <input type="text" class="search-input" id="plugin-search" placeholder="Search plugins..." value="${esc(searchQuery)}" style="flex:1;min-width:200px">
      <select class="search-select" id="plugin-category-filter">
        <option value="all"${selectedCategory === 'all' ? ' selected' : ''}>All categories</option>
        ${categoryOptions}
      </select>
      <select class="search-select" id="plugin-source-filter">
        <option value="all"${selectedSource === 'all' ? ' selected' : ''}>All Sources</option>
        <option value="claude"${selectedSource === 'claude' ? ' selected' : ''}>Claude Code</option>
        <option value="codex"${selectedSource === 'codex' ? ' selected' : ''}>Codex</option>
      </select>
      <select class="search-select" id="plugin-status-filter">
        <option value="all">All Status</option>
        <option value="installed">Installed</option>
        <option value="available">Available</option>
      </select>
    </div>
    <div id="plugin-cards">${groupsHtml}</div>
  `;

  // Bind search and filters
  const searchInput = content.querySelector('#plugin-search') as HTMLInputElement;
  const categorySelect = content.querySelector('#plugin-category-filter') as HTMLSelectElement;
  const sourceSelect = content.querySelector('#plugin-source-filter') as HTMLSelectElement;
  const statusSelect = content.querySelector('#plugin-status-filter') as HTMLSelectElement;

  const refilter = () => {
    const source = sourceSelect?.value || 'all';
    const category = categorySelect?.value || 'all';
    const status = statusSelect?.value || 'all';
    const query = searchInput?.value || '';
    // Persist filter selections
    setState({ pluginSourceFilter: source, pluginCategoryFilter: category, pluginStatusFilter: status } as any);
    // Apply all filters
    let filtered = allPlugins;
    if (status === 'installed') filtered = filtered.filter(p => p.is_installed);
    if (status === 'available') filtered = filtered.filter(p => !p.is_installed);
    if (source !== 'all') filtered = filtered.filter(p => p.source === source);
    if (query) {
      const q = query.toLowerCase();
      filtered = filtered.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.keywords.some((k: string) => k.toLowerCase().includes(q))
      );
    }
    if (category !== 'all') filtered = filtered.filter(p => p.category === category);

    // Re-render only the cards area
    const grouped: Record<string, typeof filtered> = {};
    for (const p of filtered) {
      const cat = p.category || 'uncategorized';
      if (!grouped[cat]) grouped[cat] = [];
      grouped[cat].push(p);
    }
    const sortedCats = Object.keys(grouped).sort();
    const cardsContainer = content.querySelector('#plugin-cards');
    if (cardsContainer) {
      cardsContainer.innerHTML = sortedCats.length > 0
        ? sortedCats.map(cat => `
          <div class="installed-section">
            <div class="installed-section-header">
              <span class="installed-section-label">${esc(cat)}</span>
              <span class="installed-section-count">${grouped[cat].length}</span>
            </div>
            <div class="grid">${grouped[cat].map(plugin => `
              <div class="skill-card skill-card--clickable" data-plugin-browse="${esc(plugin.name)}" data-plugin-source="${esc(plugin.source)}">
                <div class="skill-card-header">
                  <div class="skill-card-name">${esc(plugin.name)}</div>
                  ${plugin.is_installed
                    ? '<span class="skill-card-source skill-card-source--skillvault">INSTALLED</span>'
                    : '<span class="skill-card-source" style="color:var(--text-faint);border-color:var(--border)">AVAILABLE</span>'}
                </div>
                <div class="skill-card-desc">${esc(plugin.description)}</div>
                <div class="skill-card-meta">
                  <span style="color:${plugin.source === 'codex' ? '#10b981' : 'var(--accent)'}">${plugin.source === 'codex' ? 'Codex' : 'Claude Code'}</span>
                  ${plugin.category ? `<span>${esc(plugin.category)}</span>` : ''}
                  ${plugin.author_name ? `<span>${esc(plugin.author_name)}</span>` : ''}
                </div>
              </div>
            `).join('')}</div>
          </div>
        `).join('')
        : '<div class="empty-state"><div class="empty-state-text">No plugins found.</div></div>';

      // Re-bind card clicks after re-render
      bindCardClicks(content);
    }
  };

  searchInput?.addEventListener('input', refilter);
  categorySelect?.addEventListener('change', refilter);
  sourceSelect?.addEventListener('change', refilter);
  statusSelect?.addEventListener('change', refilter);

  // Bind card clicks
  bindCardClicks(content);
}

function bindCardClicks(content: HTMLElement) {
  content.querySelectorAll('[data-plugin-browse]').forEach((card) => {
    card.addEventListener('click', () => {
      const el = card as HTMLElement;
      const name = el.dataset.pluginBrowse!;
      const source = el.dataset.pluginSource || 'claude';
      setState({ selectedPluginName: name, selectedPluginSource: source });
      navigate('plugin-detail');
    });
  });
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
