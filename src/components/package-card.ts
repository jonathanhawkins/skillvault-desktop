import type { Package } from '../lib/types';

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function formatNum(n: number): string {
  return n.toLocaleString('en-US');
}

export function packageCardHtml(pkg: Package): string {
  const displayName = pkg.display_name || pkg.name;
  const authorName = pkg.author_display_name || pkg.author_id;
  const tagline = pkg.tagline ? esc(pkg.tagline) : '';

  const priceHtml =
    pkg.pricing_type === 'free'
      ? '<span class="pkg-card-price pkg-card-price--free">FREE</span>'
      : `<span class="pkg-card-price pkg-card-price--paid">$${(pkg.price_cents / 100).toFixed(2)}</span>`;

  const avatarHtml = pkg.author_avatar_url
    ? `<img class="pkg-card-avatar" src="${esc(pkg.author_avatar_url)}" alt="${esc(authorName)}" width="18" height="18" loading="lazy">`
    : '<span class="pkg-card-avatar"></span>';

  const compat = [
    { label: 'Claude Code', active: pkg.compat_claude_code },
    { label: 'Cursor', active: pkg.compat_cursor },
    { label: 'Codex', active: pkg.compat_codex },
    { label: 'Copilot', active: pkg.compat_copilot },
    { label: 'Gemini', active: pkg.compat_gemini },
  ];

  const compatHtml = compat
    .map((c) => `<span class="pkg-card-compat-dot${c.active ? ' pkg-card-compat-dot--active' : ''}" title="${c.label}"></span>`)
    .join('');

  return `
    <div class="pkg-card" data-author="${esc(pkg.author_id)}" data-name="${esc(pkg.name)}">
      <div class="pkg-card-header">
        <span class="pkg-card-category">${esc(pkg.category)}</span>
        ${priceHtml}
      </div>
      <div class="pkg-card-name">${esc(displayName)}</div>
      <div class="pkg-card-author">
        ${avatarHtml}
        <span>${esc(authorName)}</span>
      </div>
      ${tagline ? `<div class="pkg-card-tagline">${tagline}</div>` : ''}
      <div class="pkg-card-stats">
        <span class="pkg-card-stat">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          ${formatNum(pkg.download_count)}
        </span>
        <span class="pkg-card-stat">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
          ${formatNum(pkg.star_count)}
        </span>
        <span class="pkg-card-compat">${compatHtml}</span>
      </div>
    </div>
  `;
}
