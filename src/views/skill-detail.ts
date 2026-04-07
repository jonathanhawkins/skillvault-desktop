import { getState, setState } from '../lib/state';
import { getSkillDetail, uninstallSkill } from '../lib/api';
import { showToast } from '../components/toast';
import { navigate } from '../lib/router';
import { esc } from '../lib/utils';

export async function renderSkillDetail() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const skillName = state.selectedSkillName;

  if (!skillName) {
    navigate('installed');
    return;
  }

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  content.querySelector('#back-btn')?.addEventListener('click', () => {
    navigate('installed');
  });

  try {
    const skillPath = state.selectedFilePath || undefined;
    const detail = await getSkillDetail(skillName, skillPath);

    const sourceBadge = detail.source === 'skillvault'
      ? '<span class="skill-card-source skill-card-source--skillvault">SKILLVAULT</span>'
      : '<span class="skill-card-source skill-card-source--local">LOCAL</span>';

    const versionBadge = detail.installed_version
      ? `<span class="label" style="margin-left:8px">v${esc(detail.installed_version)}</span>`
      : '';

    const filesHtml = detail.files.length > 0
      ? `<div class="skill-detail-files">
          <h3 style="font-family:'Geist',sans-serif;font-size:15px;font-weight:600;color:var(--text-primary);margin:0 0 8px 0">Files</h3>
          <div style="border:1px solid var(--border);border-radius:var(--radius-md);overflow:hidden">
            ${detail.files.map(f => `
              <div style="display:flex;align-items:center;justify-content:space-between;padding:8px 12px;border-bottom:1px solid var(--border);font-size:13px">
                <div style="display:flex;align-items:center;gap:8px;min-width:0">
                  ${f.is_dir
                    ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--text-tertiary)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>'
                    : '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--text-tertiary)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>'}
                  <span style="color:var(--text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(f.name)}</span>
                </div>
                ${!f.is_dir ? `<span style="color:var(--text-tertiary);font-family:'Geist Mono',monospace;font-size:11px;flex-shrink:0;margin-left:12px">${formatSize(f.size)}</span>` : '<span style="color:var(--text-tertiary);font-size:11px;flex-shrink:0;margin-left:12px">dir</span>'}
              </div>
            `).join('')}
          </div>
        </div>`
      : '';

    const markdownHtml = detail.skill_md_content
      ? `<div class="skill-detail-content">
          <h3 style="font-family:'Geist',sans-serif;font-size:15px;font-weight:600;color:var(--text-primary);margin:0 0 8px 0">SKILL.md</h3>
          <div class="detail-description">${simpleMarkdown(detail.skill_md_content)}</div>
        </div>`
      : '<div style="color:var(--text-tertiary);font-size:13px">No SKILL.md found.</div>';

    content.innerHTML = `
      <div class="detail-back" id="back-btn">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
        Back
      </div>
      <div class="detail-hero">
        <div class="detail-meta">
          ${sourceBadge}
          ${versionBadge}
        </div>
        <h1 class="detail-name">${esc(detail.name)}</h1>
        ${detail.description ? `<p class="detail-tagline">${esc(detail.description)}</p>` : ''}
        <div style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-tertiary);margin:8px 0 16px 0">${esc(detail.path)}</div>
      </div>
      ${markdownHtml}
      <div style="margin-top:24px">${filesHtml}</div>
      <div style="margin-top:32px;padding-top:24px;border-top:1px solid var(--border)">
        <button class="btn btn--sm" id="uninstall-btn" style="color:var(--error);border-color:var(--error)">Uninstall</button>
      </div>
    `;

    // Bind events
    content.querySelector('#back-btn')?.addEventListener('click', () => {
      navigate('installed');
    });

    content.querySelector('#uninstall-btn')?.addEventListener('click', () => {
      const btn = content.querySelector('#uninstall-btn') as HTMLElement;
      if (!btn) return;
      // Two-click confirm: first click changes to "Confirm Remove", second actually removes
      if (btn.dataset.confirmed) {
        uninstallSkill(detail.name).then(() => {
          showToast(`Uninstalled "${detail.name}"`, 'success');
          setState({ localState: null });
          navigate('installed');
        }).catch((err) => {
          showToast(`Failed: ${err}`, 'error');
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
  } catch (e: any) {
    content.innerHTML = `
      <div class="detail-back" id="back-btn">Back</div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load skill: ${esc(e?.toString() || 'Unknown error')}</div>
      </div>
    `;
    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('installed'));
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function simpleMarkdown(text: string): string {
  const lines = text.split('\n');
  const html: string[] = [];
  let i = 0;

  // Skip frontmatter
  if (lines[0]?.trim() === '---') {
    i = 1;
    while (i < lines.length && lines[i].trim() !== '---') {
      i++;
    }
    i++; // skip closing ---
  }

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
      i++;
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

    // Unordered list (with continuation lines)
    if (/^\s*[-*+]\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length) {
        if (/^\s*[-*+]\s+/.test(lines[i])) {
          listItems.push(lines[i].replace(/^\s*[-*+]\s+/, ''));
          i++;
        } else if (lines[i].trim() !== '' && /^\s{2,}/.test(lines[i]) && listItems.length > 0) {
          // Continuation line (indented, non-empty) — append to last item
          listItems[listItems.length - 1] += ' ' + lines[i].trim();
          i++;
        } else {
          break;
        }
      }
      html.push(
        `<ul style="margin:8px 0;padding-left:24px;color:var(--text-secondary)">${listItems.map((li) => `<li style="margin:4px 0;line-height:1.5">${inlineMarkdown(li)}</li>`).join('')}</ul>`
      );
      continue;
    }

    // Ordered list (with continuation lines)
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length) {
        if (/^\s*\d+[.)]\s+/.test(lines[i])) {
          listItems.push(lines[i].replace(/^\s*\d+[.)]\s+/, ''));
          i++;
        } else if (lines[i].trim() !== '' && /^\s{2,}/.test(lines[i]) && listItems.length > 0) {
          // Continuation line (indented, non-empty) — append to last item
          listItems[listItems.length - 1] += ' ' + lines[i].trim();
          i++;
        } else {
          break;
        }
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

    // Regular paragraph
    const paraLines: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== '' &&
      !lines[i].trimStart().startsWith('```') &&
      !/^#{1,6}\s+/.test(lines[i]) &&
      !/^(\s*[-*_]\s*){3,}$/.test(lines[i]) &&
      !/^\s*[-*+]\s+/.test(lines[i]) &&
      !/^\s*\d+[.)]\s+/.test(lines[i]) &&
      !/^>\s?/.test(lines[i])
    ) {
      paraLines.push(lines[i]);
      i++;
    }
    if (paraLines.length > 0) {
      html.push(
        `<p style="margin:8px 0;line-height:1.6;color:var(--text-secondary)">${paraLines.map((l) => inlineMarkdown(l)).join('<br>')}</p>`
      );
    }
  }

  return html.join('');
}

function inlineMarkdown(text: string): string {
  // Escape HTML first, then apply markdown formatting
  // Protect inline code spans by extracting them before escaping
  const codeSpans: string[] = [];
  let escaped = text.replace(/`([^`]+)`/g, (_match, code) => {
    const idx = codeSpans.length;
    codeSpans.push(`<code style="background:var(--bg-secondary);padding:2px 6px;border-radius:4px;font-family:'Geist Mono',monospace;font-size:0.9em">${esc(code)}</code>`);
    return `\x00CODE${idx}\x00`;
  });

  // Escape HTML in the remaining text
  escaped = esc(escaped);

  // Apply markdown formatting on escaped text
  escaped = escaped
    .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" style="max-width:100%;height:auto;vertical-align:middle">')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener" style="color:var(--accent);text-decoration:none">$1</a>')
    .replace(/\*\*\*(.+?)\*\*\*/g, '<strong style="color:var(--text-primary)"><em>$1</em></strong>')
    .replace(/\*\*(.+?)\*\*/g, '<strong style="color:var(--text-primary)">$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/~~(.+?)~~/g, '<del>$1</del>');

  // Restore code spans
  escaped = escaped.replace(/\x00CODE(\d+)\x00/g, (_m, idx) => codeSpans[parseInt(idx)]);

  return escaped;
}
