import { getState } from '../lib/state';
import { getPluginDetail, installPlugin, uninstallPlugin, listProjects } from '../lib/api';
import { showToast } from '../components/toast';
import { navigate } from '../lib/router';
import { esc } from '../lib/utils';
import type { PluginDetail } from '../lib/types';

export async function renderPluginDetail() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const pluginName = state.selectedPluginName;
  const pluginSource = state.selectedPluginSource || 'claude';

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  content.querySelector('#back-btn')?.addEventListener('click', () => navigate('plugins'));

  let detail: PluginDetail;
  try {
    detail = await getPluginDetail(pluginName, pluginSource);
  } catch (e: any) {
    content.innerHTML = `
      <div class="detail-back" id="back-btn">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
        Back
      </div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load plugin: ${esc(e?.toString() || 'Unknown error')}</div>
      </div>
    `;
    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('plugins'));
    return;
  }

  const badge = detail.is_installed
    ? '<span class="skill-card-source skill-card-source--skillvault" style="font-size:12px;padding:4px 10px">INSTALLED</span>'
    : '<span class="skill-card-source" style="color:var(--text-faint);border-color:var(--border);font-size:12px;padding:4px 10px">AVAILABLE</span>';

  const authorHtml = detail.author_name
    ? `<div style="margin:12px 0;color:var(--text-secondary)">
        <span style="color:var(--text-faint)">Author:</span>
        ${detail.author_url
          ? `<a href="${esc(detail.author_url)}" target="_blank" rel="noopener" style="color:var(--accent);margin-left:6px">${esc(detail.author_name)}</a>`
          : `<span style="margin-left:6px">${esc(detail.author_name)}</span>`}
      </div>`
    : '';

  const keywordsHtml = detail.keywords.length > 0
    ? `<div style="display:flex;flex-wrap:wrap;gap:6px;margin:12px 0">${detail.keywords.map(k =>
        `<span style="background:var(--bg-secondary);color:var(--text-faint);padding:3px 10px;border-radius:var(--radius-sm);font-size:12px;font-family:'Geist Mono',monospace">${esc(k)}</span>`
      ).join('')}</div>`
    : '';

  const categoryBadge = detail.category
    ? `<span style="background:var(--bg-secondary);color:var(--text-secondary);padding:3px 10px;border-radius:var(--radius-sm);font-size:12px">${esc(detail.category)}</span>`
    : '';

  const sourceBadge = detail.source === 'codex'
    ? '<span style="background:rgba(16,185,129,0.12);color:#10b981;padding:3px 10px;border-radius:var(--radius-sm);font-size:12px;font-weight:600">Codex</span>'
    : '<span style="background:rgba(var(--accent-rgb,217,119,6),0.12);color:var(--accent);padding:3px 10px;border-radius:var(--radius-sm);font-size:12px;font-weight:600">Claude Code</span>';

  const installedInfoHtml = detail.is_installed
    ? `<div style="margin:16px 0;padding:16px;background:var(--bg-secondary);border-radius:var(--radius-md)">
        <div style="font-size:13px;font-weight:600;color:var(--text-primary);margin-bottom:8px">Installation Info</div>
        ${detail.installed_version ? `<div style="font-size:12px;color:var(--text-secondary);margin:4px 0"><span style="color:var(--text-faint)">Version:</span> ${esc(detail.installed_version)}</div>` : ''}
        ${detail.installed_at ? `<div style="font-size:12px;color:var(--text-secondary);margin:4px 0"><span style="color:var(--text-faint)">Installed:</span> ${esc(detail.installed_at)}</div>` : ''}
        ${detail.install_path ? `<div style="font-size:12px;color:var(--text-secondary);margin:4px 0;font-family:'Geist Mono',monospace"><span style="color:var(--text-faint);font-family:'Geist',sans-serif">Path:</span> ${esc(detail.install_path)}</div>` : ''}
      </div>`
    : '';

  // Build install/uninstall action section
  let actionHtml = '';
  if (detail.is_installed) {
    actionHtml = `
      <div style="margin:20px 0;display:flex;align-items:center;gap:12px">
        <button class="btn btn--sm" id="uninstall-btn" style="color:var(--error);border-color:var(--error)">Uninstall</button>
      </div>`;
  } else {
    actionHtml = `
      <div style="margin:20px 0;display:flex;align-items:center;gap:0;position:relative">
        <button class="btn btn--primary btn--sm" id="install-btn" style="border-top-right-radius:0;border-bottom-right-radius:0;padding:8px 16px">Install</button>
        <button class="btn btn--primary btn--sm" id="install-dropdown-btn" style="border-top-left-radius:0;border-bottom-left-radius:0;border-left:1px solid rgba(0,0,0,0.2);padding:8px 8px;min-width:0">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
        </button>
        <div id="install-dropdown" style="display:none;position:absolute;top:100%;left:0;margin-top:4px;min-width:220px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:var(--radius-md);box-shadow:0 8px 24px rgba(0,0,0,0.3);z-index:100;overflow:hidden">
          <div id="install-dropdown-items" style="max-height:260px;overflow-y:auto"></div>
        </div>
      </div>`;
  }

  const homepageBtn = detail.homepage
    ? `<button class="btn btn--sm" id="homepage-btn" style="margin-top:16px">View Homepage</button>`
    : '';

  const readmeHtml = detail.readme
    ? `<div style="margin-top:24px;border-top:1px solid var(--border);padding-top:24px">
        <div style="font-size:15px;font-weight:600;color:var(--text-primary);margin-bottom:12px">README</div>
        <div class="detail-description">${simpleMarkdown(detail.readme, detail.source === 'codex' ? `https://raw.githubusercontent.com/openai/plugins/main/plugins/${detail.name}` : `https://raw.githubusercontent.com/anthropics/claude-plugins-official/main/plugins/${detail.name}`)}</div>
      </div>`
    : '';

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back
    </div>
    <div style="margin-bottom:24px">
      <div style="display:flex;align-items:center;gap:12px;margin-bottom:8px">
        <h1 class="h1">${esc(detail.name)}</h1>
        ${badge}
      </div>
      <div style="color:var(--text-secondary);font-size:14px;line-height:1.5;margin-bottom:8px">${esc(detail.description)}</div>
      <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
        ${sourceBadge}
        ${categoryBadge}
      </div>
      ${authorHtml}
      ${keywordsHtml}
      ${actionHtml}
      ${installedInfoHtml}
      ${homepageBtn}
    </div>
    ${readmeHtml}
  `;

  // --- Event bindings ---

  content.querySelector('#back-btn')?.addEventListener('click', () => navigate('plugins'));

  if (detail.homepage) {
    content.querySelector('#homepage-btn')?.addEventListener('click', async () => {
      try {
        const { open } = window.__TAURI__.shell;
        await open(detail.homepage!);
      } catch {
        // Ignore if shell open fails
      }
    });
  }

  // --- Install button logic ---
  if (!detail.is_installed) {
    const installBtn = content.querySelector('#install-btn') as HTMLButtonElement | null;
    const dropdownBtn = content.querySelector('#install-dropdown-btn') as HTMLButtonElement | null;
    const dropdown = content.querySelector('#install-dropdown') as HTMLElement | null;
    const dropdownItems = content.querySelector('#install-dropdown-items') as HTMLElement | null;

    // Default install (global/user scope) on main button click
    installBtn?.addEventListener('click', async () => {
      await doInstall(detail, null, installBtn, dropdownBtn);
    });

    // Toggle dropdown
    dropdownBtn?.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (!dropdown || !dropdownItems) return;

      if (dropdown.style.display === 'none') {
        // Populate dropdown
        const itemStyle = 'padding:10px 14px;font-size:13px;color:var(--text-primary);cursor:pointer;border-bottom:1px solid var(--border);transition:background 0.1s';
        const hoverIn = "this.style.background='var(--bg-hover)'";
        const hoverOut = "this.style.background='transparent'";

        let items = `<div class="install-option" data-scope="user" style="${itemStyle}" onmouseover="${hoverIn}" onmouseout="${hoverOut}">
          <div style="font-weight:500">Global</div>
          <div style="font-size:11px;color:var(--text-faint);margin-top:2px">Install for all projects (user scope)</div>
        </div>`;

        // Load projects
        try {
          const projects = await listProjects();
          for (const proj of projects) {
            items += `<div class="install-option" data-scope="${esc(proj.path)}" style="${itemStyle}" onmouseover="${hoverIn}" onmouseout="${hoverOut}">
              <div style="font-weight:500">${esc(proj.name)}</div>
              <div style="font-size:11px;color:var(--text-faint);margin-top:2px;font-family:'Geist Mono',monospace;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(proj.path)}</div>
            </div>`;
          }
        } catch {
          // Ignore errors loading projects
        }

        // Choose directory option
        items += `<div class="install-option" data-scope="__choose__" style="${itemStyle};border-bottom:none" onmouseover="${hoverIn}" onmouseout="${hoverOut}">
          <div style="font-weight:500;color:var(--accent)">Choose Directory...</div>
          <div style="font-size:11px;color:var(--text-faint);margin-top:2px">Pick a project folder</div>
        </div>`;

        dropdownItems.innerHTML = items;
        dropdown.style.display = 'block';

        // Bind click handlers on dropdown items
        dropdownItems.querySelectorAll('.install-option').forEach((el) => {
          el.addEventListener('click', async (ev) => {
            ev.stopPropagation();
            dropdown.style.display = 'none';

            const scope = (el as HTMLElement).dataset.scope || 'user';

            if (scope === '__choose__') {
              try {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({ directory: true, title: 'Choose project directory' });
                if (selected) {
                  await doInstall(detail, selected as string, installBtn, dropdownBtn);
                }
              } catch {
                // User cancelled or dialog error
              }
              return;
            }

            await doInstall(detail, scope === 'user' ? null : scope, installBtn, dropdownBtn);
          });
        });
      } else {
        dropdown.style.display = 'none';
      }
    });

    // Close dropdown on outside click
    document.addEventListener('click', () => {
      if (dropdown) dropdown.style.display = 'none';
    }, { once: false });
  }

  // --- Uninstall button logic (two-click confirm) ---
  if (detail.is_installed) {
    content.querySelector('#uninstall-btn')?.addEventListener('click', () => {
      const btn = content.querySelector('#uninstall-btn') as HTMLElement;
      if (!btn) return;

      if (btn.dataset.confirmed) {
        btn.textContent = 'Removing...';
        btn.style.pointerEvents = 'none';
        uninstallPlugin(detail.name, detail.source).then(() => {
          showToast(`Uninstalled "${detail.name}"`, 'success');
          navigate('plugins');
        }).catch((err) => {
          showToast(`Failed: ${err}`, 'error');
          btn.textContent = 'Uninstall';
          btn.style.pointerEvents = '';
          btn.style.background = '';
          btn.style.borderColor = '';
          btn.style.color = '';
          delete btn.dataset.confirmed;
        });
      } else {
        btn.dataset.confirmed = 'true';
        btn.textContent = 'Confirm Remove';
        btn.style.background = 'var(--error)';
        btn.style.borderColor = 'var(--error)';
        btn.style.color = '#fff';
        setTimeout(() => {
          if (btn) {
            delete btn.dataset.confirmed;
            btn.textContent = 'Uninstall';
            btn.style.background = '';
            btn.style.borderColor = '';
            btn.style.color = '';
          }
        }, 3000);
      }
    });
  }
}

async function doInstall(
  detail: PluginDetail,
  scope: string | null,
  installBtn: HTMLButtonElement | null,
  dropdownBtn: HTMLButtonElement | null,
) {
  if (installBtn) {
    installBtn.textContent = 'Installing...';
    installBtn.disabled = true;
  }
  if (dropdownBtn) dropdownBtn.disabled = true;

  try {
    const msg = await installPlugin(detail.name, detail.source, scope);
    showToast(msg, 'success');
    // Re-render to show installed state
    renderPluginDetail();
  } catch (err: any) {
    showToast(`Install failed: ${err}`, 'error');
    if (installBtn) {
      installBtn.textContent = 'Install';
      installBtn.disabled = false;
    }
    if (dropdownBtn) dropdownBtn.disabled = false;
  }
}

function simpleMarkdown(text: string, baseUrl?: string): string {
  // Rewrite relative image/link URLs to absolute GitHub raw URLs
  let processed = text;
  if (baseUrl) {
    // Fix relative image paths: ![alt](./path) or ![alt](path)
    processed = processed.replace(/!\[([^\]]*)\]\((?!https?:\/\/)([^)]+)\)/g, (_, alt, path) => {
      const cleanPath = path.replace(/^\.\//, '');
      return `![${alt}](${baseUrl}/${cleanPath})`;
    });
    // Fix relative link paths: [text](./path) but not [text](http...)
    processed = processed.replace(/\[([^\]]+)\]\((?!https?:\/\/)(?!#)([^)]+)\)/g, (_, text, path) => {
      const cleanPath = path.replace(/^\.\//, '');
      return `[${text}](${baseUrl}/${cleanPath})`;
    });
    // Fix HTML <img src="relative"> tags
    processed = processed.replace(/src="(?!https?:\/\/)([^"]+)"/g, (_, path) => {
      const cleanPath = path.replace(/^\.\//, '');
      return `src="${baseUrl}/${cleanPath}"`;
    });
  }

  const lines = processed.split('\n');
  const html: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.trimStart().startsWith('```')) {
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].trimStart().startsWith('```')) {
        codeLines.push(lines[i]);
        i++;
      }
      i++;
      html.push(`<pre style="background:var(--bg-secondary);padding:12px;border-radius:var(--radius-md);overflow-x:auto;font-family:'Geist Mono',monospace;font-size:13px;color:var(--text-primary);margin:12px 0;line-height:1.5"><code>${esc(codeLines.join('\n'))}</code></pre>`);
      continue;
    }

    if (line.trim() === '') { i++; continue; }

    if (/^(\s*[-*_]\s*){3,}$/.test(line)) {
      html.push('<hr style="border:none;border-top:1px solid var(--border);margin:16px 0">');
      i++; continue;
    }

    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      const level = headingMatch[1].length;
      const sizes: Record<number, string> = { 1: '24px', 2: '20px', 3: '17px', 4: '15px', 5: '14px', 6: '13px' };
      html.push(`<h${level} style="font-family:'Geist',sans-serif;font-size:${sizes[level]};font-weight:600;color:var(--text-primary);margin:20px 0 8px;line-height:1.3">${inlineMd(headingMatch[2])}</h${level}>`);
      i++; continue;
    }

    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*[-*+]\s+/, ''));
        i++;
      }
      html.push(`<ul style="margin:8px 0;padding-left:24px;color:var(--text-secondary)">${items.map(li => `<li style="margin:4px 0;line-height:1.5">${inlineMd(li)}</li>`).join('')}</ul>`);
      continue;
    }

    // Paragraph
    const paraLines: string[] = [];
    while (i < lines.length && lines[i].trim() !== '' && !lines[i].trimStart().startsWith('```') && !/^#{1,6}\s+/.test(lines[i]) && !/^\s*[-*+]\s+/.test(lines[i])) {
      paraLines.push(lines[i]);
      i++;
    }
    if (paraLines.length > 0) {
      html.push(`<p style="margin:8px 0;line-height:1.6;color:var(--text-secondary)">${paraLines.map(l => inlineMd(l)).join('<br>')}</p>`);
    }
  }

  return html.join('');
}

function inlineMd(text: string): string {
  // Protect inline code spans, then escape HTML, then apply formatting
  const codeSpans: string[] = [];
  let s = text.replace(/`([^`]+)`/g, (_m, code) => {
    const idx = codeSpans.length;
    codeSpans.push(`<code style="background:var(--bg-secondary);padding:2px 6px;border-radius:4px;font-family:'Geist Mono',monospace;font-size:0.9em">${esc(code)}</code>`);
    return `\x00CODE${idx}\x00`;
  });
  s = esc(s);
  s = s
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener" style="color:var(--accent)">$1</a>')
    .replace(/\*\*(.+?)\*\*/g, '<strong style="color:var(--text-primary)">$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>');
  s = s.replace(/\x00CODE(\d+)\x00/g, (_m, idx) => codeSpans[parseInt(idx)]);
  return s;
}
