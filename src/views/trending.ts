import { getState, setState } from '../lib/state';
import { getTrending } from '../lib/api';
import { packageCardHtml } from '../components/package-card';
import { navigate } from '../lib/router';

export async function renderTrending() {
  const content = document.getElementById('content');
  if (!content) return;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Trending</h1>
      </div>
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  try {
    const packages = await getTrending();
    setState({ trendingPackages: packages });

    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title">
          <h1 class="h1">Trending</h1>
        </div>
      </div>
      <div class="grid">${packages.map(packageCardHtml).join('')}</div>
    `;

    content.querySelectorAll('.pkg-card').forEach((card) => {
      card.addEventListener('click', () => {
        const author = (card as HTMLElement).dataset.author!;
        const name = (card as HTMLElement).dataset.name!;
        setState({ selectedAuthor: author, selectedName: name, selectedPackage: null });
        navigate('detail');
      });
    });
  } catch (e: any) {
    content.innerHTML = `
      <div class="view-header">
        <div class="view-header-title"><h1 class="h1">Trending</h1></div>
      </div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load trending: ${e?.toString()}</div>
      </div>
    `;
  }
}
