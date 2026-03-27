import { setState } from '../lib/state';
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

  renderPluginList(content, plugins, categories, '', 'all');
}

function renderPluginList(
  content: HTMLElement,
  allPlugins: MarketplacePlugin[],
  categories: string[],
  searchQuery: string,
  selectedCategory: string
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
          <div class="skill-card skill-card--clickable" data-plugin-browse="${esc(plugin.name)}">
            <div class="skill-card-header">
              <div class="skill-card-name">${esc(plugin.name)}</div>
              ${plugin.is_installed
                ? '<span class="skill-card-source skill-card-source--skillvault">INSTALLED</span>'
                : '<span class="skill-card-source" style="color:var(--text-faint);border-color:var(--border)">AVAILABLE</span>'}
            </div>
            <div class="skill-card-desc">${esc(plugin.description)}</div>
            <div class="skill-card-meta">
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
    <div style="display:flex;gap:12px;margin-bottom:24px">
      <input type="text" class="search-input" id="plugin-search" placeholder="Search plugins..." value="${esc(searchQuery)}" style="flex:1">
      <select class="search-select" id="plugin-category-filter">
        <option value="all"${selectedCategory === 'all' ? ' selected' : ''}>All categories</option>
        ${categoryOptions}
      </select>
    </div>
    ${groupsHtml}
  `;

  // Bind search
  const searchInput = content.querySelector('#plugin-search') as HTMLInputElement;
  const categorySelect = content.querySelector('#plugin-category-filter') as HTMLSelectElement;

  searchInput?.addEventListener('input', () => {
    renderPluginList(content, allPlugins, categories, searchInput.value, categorySelect.value);
  });

  categorySelect?.addEventListener('change', () => {
    renderPluginList(content, allPlugins, categories, searchInput.value, categorySelect.value);
  });

  // Bind card clicks
  content.querySelectorAll('[data-plugin-browse]').forEach((card) => {
    card.addEventListener('click', () => {
      const name = (card as HTMLElement).dataset.pluginBrowse!;
      setState({ selectedPluginName: name });
      navigate('plugin-detail');
    });
  });
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
