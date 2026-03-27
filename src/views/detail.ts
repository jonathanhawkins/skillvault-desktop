import { getState, setState } from '../lib/state';
import { getPackage, installPackage, listProjects } from '../lib/api';
import { showToast } from '../components/toast';
import { navigate } from '../lib/router';

export async function renderDetail() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const { selectedAuthor, selectedName } = state;

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  content.querySelector('#back-btn')?.addEventListener('click', () => {
    navigate('browse');
  });

  try {
    let pkg = state.selectedPackage;
    if (!pkg) {
      pkg = await getPackage(selectedAuthor, selectedName);
      setState({ selectedPackage: pkg });
    }

    // Check if already installed
    const installed = state.localState?.skills.find(
      (s) => s.package_id === `${pkg!.author_id}/${pkg!.name}`
    );

    const compat = [
      { label: 'Claude Code', active: pkg.compat_claude_code },
      { label: 'Cursor', active: pkg.compat_cursor },
      { label: 'Codex', active: pkg.compat_codex },
      { label: 'Copilot', active: pkg.compat_copilot },
      { label: 'Gemini', active: pkg.compat_gemini },
    ];

    const compatHtml = compat
      .filter((c) => c.active)
      .map((c) => `<span class="badge">${esc(c.label)}</span>`)
      .join(' ');

    const priceHtml =
      pkg.pricing_type === 'free'
        ? '<span class="badge" style="color:var(--success);border-color:var(--success)">FREE</span>'
        : `<span class="badge" style="color:var(--accent);border-color:var(--accent)">$${(pkg.price_cents / 100).toFixed(2)}</span>`;

    const installBtnHtml = installed
      ? '<button class="btn btn--sm" disabled>Installed</button>'
      : `<div class="install-dropdown-wrap" style="position:relative;display:inline-block">
          <button class="btn btn--primary btn--sm" id="install-btn">Install</button>
          <button class="btn btn--primary btn--sm" id="install-chevron" style="padding:4px 6px;margin-left:-1px;border-left:1px solid rgba(0,0,0,0.2)" aria-label="Choose install location">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </button>
          <div id="install-menu" style="display:none;position:absolute;top:100%;left:0;margin-top:4px;min-width:280px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:var(--radius-md);padding:4px 0;z-index:100;box-shadow:0 8px 24px rgba(0,0,0,0.3)">
            <div class="install-menu-item" data-location="global" style="padding:8px 12px;cursor:pointer;font-size:13px;color:var(--text-primary)">
              <div style="font-weight:500">Global</div>
              <div style="font-size:11px;color:var(--text-tertiary);margin-top:2px">~/.claude/skills/ — available to all projects</div>
            </div>
            <div id="install-projects-list"></div>
          </div>
        </div>`;

    content.innerHTML = `
      <div class="detail-back" id="back-btn">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
        Back
      </div>
      <div class="detail-hero">
        <div class="detail-meta">
          <span class="label">${esc(pkg.category)}</span>
          ${priceHtml}
          <span class="label">v${esc(pkg.current_version)}</span>
        </div>
        <h1 class="detail-name">${esc(pkg.display_name || pkg.name)}</h1>
        <div class="pkg-card-author" style="margin-bottom:12px">
          ${pkg.author_avatar_url ? `<img class="pkg-card-avatar" src="${esc(pkg.author_avatar_url)}" width="20" height="20">` : ''}
          <span>${esc(pkg.author_display_name || pkg.author_id)}</span>
        </div>
        ${pkg.tagline ? `<p class="detail-tagline">${esc(pkg.tagline)}</p>` : ''}
        <div class="detail-actions">
          ${installBtnHtml}
          ${pkg.repo_url ? `<a class="btn btn--sm" href="${esc(pkg.repo_url)}" target="_blank" rel="noopener">Repository</a>` : ''}
        </div>
      </div>
      <div class="detail-stats">
        <div class="detail-stat">
          <span class="detail-stat-value">${formatNum(pkg.download_count)}</span>
          <span class="detail-stat-label">Downloads</span>
        </div>
        <div class="detail-stat">
          <span class="detail-stat-value">${formatNum(pkg.star_count)}</span>
          <span class="detail-stat-label">Stars</span>
        </div>
        ${pkg.review_count ? `<div class="detail-stat">
          <span class="detail-stat-value">${pkg.avg_rating?.toFixed(1) || '—'}</span>
          <span class="detail-stat-label">${pkg.review_count} Reviews</span>
        </div>` : ''}
        ${pkg.license ? `<div class="detail-stat">
          <span class="detail-stat-value">${esc(pkg.license)}</span>
          <span class="detail-stat-label">License</span>
        </div>` : ''}
      </div>
      <div style="margin-bottom:16px">${compatHtml}</div>
      ${pkg.description ? `<div class="detail-description">${simpleMarkdown(pkg.description)}</div>` : ''}
    `;

    // Bind events
    content.querySelector('#back-btn')?.addEventListener('click', () => {
      navigate('browse');
    });

    // Install dropdown logic
    const chevronBtn = content.querySelector('#install-chevron');
    const installMenu = content.querySelector('#install-menu') as HTMLElement | null;
    const installBtn = content.querySelector('#install-btn');
    const projectsList = content.querySelector('#install-projects-list');

    // Populate projects list when chevron is clicked
    if (chevronBtn && installMenu && projectsList) {
      chevronBtn.addEventListener('click', async () => {
        if (installMenu.style.display === 'block') {
          installMenu.style.display = 'none';
          return;
        }
        // Load projects
        try {
          const projects = await listProjects();
          if (projects.length > 0) {
            projectsList.innerHTML =
              '<div style="border-top:1px solid var(--border);margin:4px 0"></div>' +
              projects.map(p =>
                `<div class="install-menu-item" data-location="${esc(p.path)}" style="padding:8px 12px;cursor:pointer;font-size:13px;color:var(--text-primary)">
                  <div style="font-weight:500">${esc(p.name)}</div>
                  <div style="font-size:11px;color:var(--text-tertiary);margin-top:2px">${esc(p.path)}</div>
                </div>`
              ).join('');
          } else {
            projectsList.innerHTML =
              '<div style="border-top:1px solid var(--border);margin:4px 0"></div>' +
              '<div style="padding:8px 12px;font-size:12px;color:var(--text-muted)">No projects found in ~/.claude/projects/</div>';
          }
          // Always add "Choose Directory..." option
          projectsList.innerHTML +=
            '<div style="border-top:1px solid var(--border);margin:4px 0"></div>' +
            '<div class="install-menu-item" data-location="__choose__" style="padding:8px 12px;cursor:pointer;font-size:13px;color:var(--accent)">' +
            '<div style="display:flex;align-items:center;gap:6px"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg> Choose Directory...</div>' +
            '</div>';
        } catch {
          projectsList.innerHTML = '';
        }
        installMenu.style.display = 'block';

        // Add hover styles to menu items
        installMenu.querySelectorAll('.install-menu-item').forEach(item => {
          (item as HTMLElement).addEventListener('mouseenter', () => {
            (item as HTMLElement).style.background = 'var(--bg-tertiary)';
          });
          (item as HTMLElement).addEventListener('mouseleave', () => {
            (item as HTMLElement).style.background = 'transparent';
          });
        });

        // Bind click on each menu item
        installMenu.querySelectorAll('.install-menu-item').forEach(item => {
          item.addEventListener('click', async () => {
            const location = (item as HTMLElement).dataset.location!;
            installMenu.style.display = 'none';

            if (location === '__choose__') {
              // Open native directory picker
              try {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({ directory: true, multiple: false, title: 'Choose project directory' });
                if (selected) {
                  doInstall(selected as string);
                }
              } catch (err: any) {
                showToast(`Failed to open picker: ${err}`, 'error');
              }
            } else {
              doInstall(location === 'global' ? null : location);
            }
          });
        });
      });

      // Close menu on outside click
      document.addEventListener('click', (e) => {
        if (!chevronBtn.contains(e.target as Node) && !installMenu.contains(e.target as Node)) {
          installMenu.style.display = 'none';
        }
      });
    }

    // Default install button: install globally
    if (installBtn) {
      installBtn.addEventListener('click', () => doInstall(null));
    }

    async function doInstall(installPath: string | null) {
      const btn = content!.querySelector('#install-btn') as HTMLButtonElement | null;
      const chevron = content!.querySelector('#install-chevron') as HTMLButtonElement | null;
      if (btn) { btn.disabled = true; btn.textContent = 'Installing...'; }
      if (chevron) { chevron.disabled = true; }
      if (installMenu) { installMenu.style.display = 'none'; }
      try {
        const msg = await installPackage(pkg!.author_id, pkg!.name, installPath);
        showToast(msg, 'success');
        setState({ localState: null }); // Force rescan
        if (btn) { btn.textContent = 'Installed'; }
      } catch (err: any) {
        showToast(`Install failed: ${err}`, 'error');
        if (btn) { btn.disabled = false; btn.textContent = 'Install'; }
        if (chevron) { chevron.disabled = false; }
      }
    }
  } catch (e: any) {
    content.innerHTML = `
      <div class="detail-back" id="back-btn">Back</div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load package: ${e?.toString()}</div>
      </div>
    `;
    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('browse'));
  }
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function formatNum(n: number): string {
  return n.toLocaleString('en-US');
}

function simpleMarkdown(text: string): string {
  const lines = text.split('\n');
  const html: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Fenced code block
    if (line.trimStart().startsWith('```')) {
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].trimStart().startsWith('```')) {
        codeLines.push(lines[i]);
        i++;
      }
      i++; // skip closing ```
      html.push(
        `<pre style="background:var(--bg-secondary);padding:12px;border-radius:var(--radius-md);overflow-x:auto;font-family:'Geist Mono',monospace;font-size:13px;color:var(--text-primary);margin:12px 0;line-height:1.5"><code>${esc(codeLines.join('\n'))}</code></pre>`
      );
      continue;
    }

    // Blank line
    if (line.trim() === '') {
      i++;
      continue;
    }

    // Horizontal rule
    if (/^(\s*[-*_]\s*){3,}$/.test(line)) {
      html.push('<hr style="border:none;border-top:1px solid var(--border);margin:16px 0">');
      i++;
      continue;
    }

    // Headings
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      const level = headingMatch[1].length;
      const sizes: Record<number, string> = { 1: '24px', 2: '20px', 3: '17px', 4: '15px', 5: '14px', 6: '13px' };
      const margins: Record<number, string> = { 1: '24px 0 12px', 2: '20px 0 10px', 3: '16px 0 8px', 4: '14px 0 6px', 5: '12px 0 6px', 6: '12px 0 6px' };
      html.push(
        `<h${level} style="font-family:'Geist',sans-serif;font-size:${sizes[level]};font-weight:600;color:var(--text-primary);margin:${margins[level]};line-height:1.3">${inlineMarkdown(headingMatch[2])}</h${level}>`
      );
      i++;
      continue;
    }

    // Image-only line (badges etc): ![alt](url)
    if (/^\s*!\[.*?\]\(.+?\)\s*$/.test(line)) {
      const imgMatch = line.match(/!\[([^\]]*)\]\(([^)]+)\)/);
      if (imgMatch) {
        html.push(
          `<p style="margin:8px 0"><img src="${esc(imgMatch[2])}" alt="${esc(imgMatch[1])}" style="max-width:100%;height:auto"></p>`
        );
        i++;
        continue;
      }
    }

    // Table
    if (line.includes('|') && i + 1 < lines.length && /^\s*\|?\s*[-:]+/.test(lines[i + 1])) {
      const tableLines: string[] = [];
      while (i < lines.length && lines[i].includes('|')) {
        tableLines.push(lines[i]);
        i++;
      }
      html.push(renderTable(tableLines));
      continue;
    }

    // Unordered list
    if (/^\s*[-*+]\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        listItems.push(lines[i].replace(/^\s*[-*+]\s+/, ''));
        i++;
      }
      html.push(
        `<ul style="margin:8px 0;padding-left:24px;color:var(--text-secondary)">${listItems.map((li) => `<li style="margin:4px 0;line-height:1.5">${inlineMarkdown(li)}</li>`).join('')}</ul>`
      );
      continue;
    }

    // Ordered list
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length && /^\s*\d+[.)]\s+/.test(lines[i])) {
        listItems.push(lines[i].replace(/^\s*\d+[.)]\s+/, ''));
        i++;
      }
      html.push(
        `<ol style="margin:8px 0;padding-left:24px;color:var(--text-secondary)">${listItems.map((li) => `<li style="margin:4px 0;line-height:1.5">${inlineMarkdown(li)}</li>`).join('')}</ol>`
      );
      continue;
    }

    // Blockquote
    if (/^>\s?/.test(line)) {
      const quoteLines: string[] = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        quoteLines.push(lines[i].replace(/^>\s?/, ''));
        i++;
      }
      html.push(
        `<blockquote style="margin:12px 0;padding:8px 16px;border-left:3px solid var(--accent);color:var(--text-secondary);font-style:italic">${quoteLines.map((l) => inlineMarkdown(l)).join('<br>')}</blockquote>`
      );
      continue;
    }

    // Regular paragraph — collect consecutive non-special lines
    const paraLines: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== '' &&
      !lines[i].trimStart().startsWith('```') &&
      !/^#{1,6}\s+/.test(lines[i]) &&
      !/^(\s*[-*_]\s*){3,}$/.test(lines[i]) &&
      !/^\s*[-*+]\s+/.test(lines[i]) &&
      !/^\s*\d+[.)]\s+/.test(lines[i]) &&
      !/^>\s?/.test(lines[i]) &&
      !(lines[i].includes('|') && i + 1 < lines.length && /^\s*\|?\s*[-:]+/.test(lines[i + 1]))
    ) {
      paraLines.push(lines[i]);
      i++;
    }
    if (paraLines.length > 0) {
      html.push(
        `<p style="margin:8px 0;line-height:1.6;color:var(--text-secondary)">${inlineMarkdown(paraLines.join('\n').replace(/\n/g, '<br>'))}</p>`
      );
    }
  }

  return html.join('');
}

function inlineMarkdown(text: string): string {
  return text
    // Images (must come before links)
    .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" style="max-width:100%;height:auto;vertical-align:middle">')
    // Links
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener" style="color:var(--accent);text-decoration:none">$1</a>')
    // Bold + italic
    .replace(/\*\*\*(.+?)\*\*\*/g, '<strong style="color:var(--text-primary)"><em>$1</em></strong>')
    // Bold
    .replace(/\*\*(.+?)\*\*/g, '<strong style="color:var(--text-primary)">$1</strong>')
    // Italic
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // Strikethrough
    .replace(/~~(.+?)~~/g, '<del>$1</del>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code style="background:var(--bg-secondary);padding:2px 6px;border-radius:4px;font-family:\'Geist Mono\',monospace;font-size:0.9em">$1</code>');
}

function renderTable(tableLines: string[]): string {
  const parseCells = (row: string): string[] => {
    const trimmed = row.trim();
    const raw = trimmed.split('|');
    // If line starts with |, first element is empty; if ends with |, last is empty
    if (trimmed.startsWith('|')) raw.shift();
    if (trimmed.endsWith('|')) raw.pop();
    return raw.map((c) => c.trim());
  };

  if (tableLines.length < 2) return '';

  const headers = parseCells(tableLines[0]);
  // Skip separator row (index 1)
  const bodyRows = tableLines.slice(2).map(parseCells);

  const thCells = headers
    .map((h) => `<th style="padding:8px 12px;text-align:left;font-weight:600;color:var(--text-primary);border-bottom:2px solid var(--border);font-size:13px">${inlineMarkdown(h)}</th>`)
    .join('');

  const trRows = bodyRows
    .map(
      (cells) =>
        `<tr>${cells.map((c) => `<td style="padding:6px 12px;border-bottom:1px solid var(--border);color:var(--text-secondary);font-size:13px">${inlineMarkdown(c)}</td>`).join('')}</tr>`
    )
    .join('');

  return `<div style="overflow-x:auto;margin:12px 0"><table style="width:100%;border-collapse:collapse;border:1px solid var(--border);border-radius:var(--radius-md)"><thead><tr>${thCells}</tr></thead><tbody>${trRows}</tbody></table></div>`;
}
