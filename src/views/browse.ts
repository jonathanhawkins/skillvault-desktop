import { getState, setState } from '../lib/state';
import { searchPackages, getCategories } from '../lib/api';
import { packageCardHtml } from '../components/package-card';
import { navigate } from '../lib/router';

let categories: { category: string; count: number }[] = [];

export async function renderBrowse() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Browse</h1>
      </div>
    </div>
    <div class="browse-filters">
      <input class="search-input" id="search-input" type="text" placeholder="Search packages..." value="${esc(state.searchQuery)}">
      <select class="search-select" id="category-select">
        <option value="">All Categories</option>
      </select>
      <select class="search-select" id="sort-select">
        <option value="trending"${state.searchSort === 'trending' ? ' selected' : ''}>Trending</option>
        <option value="newest"${state.searchSort === 'newest' ? ' selected' : ''}>Newest</option>
        <option value="downloads"${state.searchSort === 'downloads' ? ' selected' : ''}>Downloads</option>
        <option value="stars"${state.searchSort === 'stars' ? ' selected' : ''}>Stars</option>
      </select>
    </div>
    <div id="results-info" class="browse-results-info"></div>
    <div id="results-grid" class="grid"></div>
    <div id="load-more" class="browse-load-more" style="display:none">
      <button class="btn btn--sm" id="load-more-btn">Load More</button>
    </div>
  `;

  // Load categories
  if (categories.length === 0) {
    try {
      categories = await getCategories();
    } catch {}
  }

  const catSelect = content.querySelector('#category-select') as HTMLSelectElement;
  categories.forEach((c) => {
    const opt = document.createElement('option');
    opt.value = c.category;
    opt.textContent = `${c.category} (${c.count})`;
    if (state.searchCategory === c.category) opt.selected = true;
    catSelect.appendChild(opt);
  });

  // Search handler
  let searchTimeout: number;
  const searchInput = content.querySelector('#search-input') as HTMLInputElement;
  const sortSelect = content.querySelector('#sort-select') as HTMLSelectElement;

  const doSearch = async (page = 1) => {
    const query = searchInput.value;
    const category = catSelect.value || null;
    const sort = sortSelect.value;

    setState({ searchQuery: query, searchCategory: category, searchSort: sort, searchPage: page });

    const grid = content.querySelector('#results-grid')!;
    const info = content.querySelector('#results-info')!;
    const loadMore = content.querySelector('#load-more')! as HTMLElement;

    if (page === 1) {
      grid.innerHTML = `<div style="grid-column:1/-1;display:flex;justify-content:center;padding:32px"><div class="spinner"></div></div>`;
    }

    try {
      const result = await searchPackages(query, category, sort, page, 20);

      if (page === 1) {
        setState({ searchResults: result.packages, searchTotal: result.total });
        grid.innerHTML = result.packages.map(packageCardHtml).join('');
      } else {
        const combined = [...getState().searchResults, ...result.packages];
        setState({ searchResults: combined, searchTotal: result.total });
        grid.innerHTML = combined.map(packageCardHtml).join('');
      }

      info.textContent = `${result.total} package${result.total !== 1 ? 's' : ''} found`;
      loadMore.style.display = result.packages.length >= 20 ? 'flex' : 'none';

      // Bind card clicks
      grid.querySelectorAll('.pkg-card').forEach((card) => {
        card.addEventListener('click', () => {
          const author = (card as HTMLElement).dataset.author!;
          const name = (card as HTMLElement).dataset.name!;
          setState({ selectedAuthor: author, selectedName: name, selectedPackage: null });
          navigate('detail');
        });
      });
    } catch (e: any) {
      grid.innerHTML = `<div style="grid-column:1/-1" class="empty-state"><div class="empty-state-text">Failed to search: ${e?.toString()}</div></div>`;
    }
  };

  searchInput.addEventListener('input', () => {
    clearTimeout(searchTimeout);
    searchTimeout = window.setTimeout(() => doSearch(1), 300);
  });

  catSelect.addEventListener('change', () => doSearch(1));
  sortSelect.addEventListener('change', () => doSearch(1));

  content.querySelector('#load-more-btn')?.addEventListener('click', () => {
    doSearch(getState().searchPage + 1);
  });

  // Initial search
  doSearch(1);
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
