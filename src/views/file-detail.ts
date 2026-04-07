import { getState } from '../lib/state';
import { readFileContent } from '../lib/api';
import { navigate } from '../lib/router';
import { esc } from '../lib/utils';

export async function renderFileDetail() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const { selectedFilePath, selectedFileTitle } = state;

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back
    </div>
    <div style="display:flex;justify-content:center;padding:64px"><div class="spinner"></div></div>
  `;

  content.querySelector('#back-btn')?.addEventListener('click', () => navigate('installed'));

  try {
    const fileContent = await readFileContent(selectedFilePath);

    const shortPath = selectedFilePath.replace(/^\/Users\/[^/]+/, '~');

    content.innerHTML = `
      <div class="detail-back" id="back-btn">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
        Back
      </div>
      <div style="margin-bottom:24px">
        <h1 class="h1" style="margin-bottom:8px">${esc(selectedFileTitle)}</h1>
        <div style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-faint)">${esc(shortPath)}</div>
      </div>
      <div class="detail-description">${renderContent(selectedFilePath, fileContent)}</div>
    `;

    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('installed'));
  } catch (e: any) {
    content.innerHTML = `
      <div class="detail-back" id="back-btn">Back</div>
      <div class="empty-state">
        <div class="empty-state-text">Failed to load: ${e?.toString()}</div>
      </div>
    `;
    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('installed'));
  }
}

function renderContent(filePath: string, content: string): string {
  if (filePath.endsWith('.md')) {
    return simpleMarkdown(content);
  }
  if (filePath.endsWith('.json')) {
    return `<pre style="background:var(--bg-secondary);padding:16px;border-radius:var(--radius-md);overflow-x:auto;font-family:'Geist Mono',monospace;font-size:13px;color:var(--text-primary);line-height:1.5"><code>${esc(content)}</code></pre>`;
  }
  // Default: show as preformatted
  return `<pre style="background:var(--bg-secondary);padding:16px;border-radius:var(--radius-md);overflow-x:auto;font-family:'Geist Mono',monospace;font-size:13px;color:var(--text-primary);line-height:1.5;white-space:pre-wrap"><code>${esc(content)}</code></pre>`;
}

function simpleMarkdown(text: string): string {
  const lines = text.split('\n');
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
