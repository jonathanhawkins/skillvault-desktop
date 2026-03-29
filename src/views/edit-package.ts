import { getState, setState } from '../lib/state';
import { updatePackage } from '../lib/api';
import { showToast } from '../components/toast';
import { navigate } from '../lib/router';
import { esc } from '../lib/utils';

const CATEGORIES = [
  'automation',
  'coding',
  'configs',
  'data',
  'database',
  'deployment',
  'design',
  'devops',
  'docs',
  'gamedev',
  'ai',
  'learning',
  'mobile',
  'monitoring',
  'productivity',
  'security',
  'testing',
  'toolkit',
  'web',
  'other',
];

export async function renderEditPackage() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const pkg = state.selectedPackage;

  if (!pkg) {
    content.innerHTML = `
      <div class="detail-back" id="back-btn">Back</div>
      <div class="empty-state">
        <div class="empty-state-text">No package selected</div>
      </div>
    `;
    content.querySelector('#back-btn')?.addEventListener('click', () => navigate('browse'));
    return;
  }

  content.innerHTML = `
    <div class="detail-back" id="back-btn">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>
      Back to ${esc(pkg.display_name || pkg.name)}
    </div>

    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Edit Package</h1>
        <p style="color:var(--text-muted);font-size:13px;margin-top:4px">${esc(pkg.author_id)}/${esc(pkg.name)}</p>
      </div>
    </div>

    <div class="settings-section" style="max-width:720px">
      <div style="margin-bottom:16px">
        <label class="settings-label">Display Name</label>
        <input class="settings-input" id="edit-display-name" type="text" value="${esc(pkg.display_name || '')}" placeholder="My Awesome Skill">
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">Tagline</label>
        <input class="settings-input" id="edit-tagline" type="text" value="${esc(pkg.tagline || '')}" placeholder="A short description of what this does">
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">Description</label>
        <div style="font-size:11px;color:var(--text-faint);margin-bottom:6px">Supports Markdown — headings, bold, links, lists, code blocks, tables</div>
        <div style="display:flex;gap:8px;margin-bottom:6px">
          <button type="button" class="btn btn--sm" id="desc-tab-write" style="padding:3px 10px;font-size:11px;background:var(--bg-tertiary)">Write</button>
          <button type="button" class="btn btn--sm" id="desc-tab-preview" style="padding:3px 10px;font-size:11px">Preview</button>
        </div>
        <div id="desc-write-pane">
          <textarea class="settings-input" id="edit-description" rows="16" style="resize:vertical;font-family:'Geist Mono',monospace;font-size:13px;line-height:1.5;min-height:200px">${esc(pkg.description || '')}</textarea>
        </div>
        <div id="desc-preview-pane" style="display:none;padding:12px 16px;background:var(--bg-secondary);border:1px solid var(--border);border-radius:var(--radius-md);min-height:200px;font-size:14px;line-height:1.6;color:var(--text-secondary);overflow-y:auto;max-height:500px"></div>
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">Category</label>
        <select class="settings-input" id="edit-category" style="appearance:auto">
          ${CATEGORIES.map(c => `<option value="${c}" ${pkg.category === c ? 'selected' : ''}>${c}</option>`).join('')}
        </select>
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">License</label>
        <input class="settings-input" id="edit-license" type="text" value="${esc(pkg.license || '')}" placeholder="MIT">
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">Repository URL</label>
        <input class="settings-input" id="edit-repo-url" type="url" value="${esc(pkg.repo_url || '')}" placeholder="https://github.com/...">
      </div>

      <div style="margin-bottom:16px">
        <label class="settings-label">Homepage URL</label>
        <input class="settings-input" id="edit-homepage-url" type="url" value="${esc(pkg.homepage_url || '')}" placeholder="https://...">
      </div>

      <div style="margin-bottom:20px">
        <label class="settings-label">Compatibility</label>
        <div style="display:flex;flex-wrap:wrap;gap:12px;margin-top:6px">
          <label style="display:flex;align-items:center;gap:6px;font-size:13px;color:var(--text-secondary);cursor:pointer">
            <input type="checkbox" id="compat-claude-code" ${pkg.compat_claude_code ? 'checked' : ''}> Claude Code
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:13px;color:var(--text-secondary);cursor:pointer">
            <input type="checkbox" id="compat-cursor" ${pkg.compat_cursor ? 'checked' : ''}> Cursor
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:13px;color:var(--text-secondary);cursor:pointer">
            <input type="checkbox" id="compat-codex" ${pkg.compat_codex ? 'checked' : ''}> Codex
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:13px;color:var(--text-secondary);cursor:pointer">
            <input type="checkbox" id="compat-copilot" ${pkg.compat_copilot ? 'checked' : ''}> Copilot
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:13px;color:var(--text-secondary);cursor:pointer">
            <input type="checkbox" id="compat-gemini" ${pkg.compat_gemini ? 'checked' : ''}> Gemini
          </label>
        </div>
      </div>

      <div style="display:flex;gap:8px">
        <button class="btn btn--primary btn--sm" id="save-btn">Save Changes</button>
        <button class="btn btn--sm" id="cancel-btn">Cancel</button>
      </div>
    </div>
  `;

  // Back / Cancel -> detail
  const goBackToDetail = () => {
    setState({ selectedPackage: pkg, selectedAuthor: pkg.author_id, selectedName: pkg.name });
    navigate('detail');
  };

  content.querySelector('#back-btn')?.addEventListener('click', goBackToDetail);
  content.querySelector('#cancel-btn')?.addEventListener('click', goBackToDetail);

  // Write / Preview tabs for description
  const writeTab = content.querySelector('#desc-tab-write') as HTMLButtonElement;
  const previewTab = content.querySelector('#desc-tab-preview') as HTMLButtonElement;
  const writePane = content.querySelector('#desc-write-pane') as HTMLElement;
  const previewPane = content.querySelector('#desc-preview-pane') as HTMLElement;

  if (writeTab && previewTab && writePane && previewPane) {
    writeTab.addEventListener('click', () => {
      writePane.style.display = '';
      previewPane.style.display = 'none';
      writeTab.style.background = 'var(--bg-tertiary)';
      previewTab.style.background = '';
    });

    previewTab.addEventListener('click', () => {
      const textarea = content.querySelector('#edit-description') as HTMLTextAreaElement;
      const md = textarea.value || '';
      writePane.style.display = 'none';
      previewPane.style.display = '';
      previewTab.style.background = 'var(--bg-tertiary)';
      writeTab.style.background = '';
      // Simple markdown rendering
      previewPane.innerHTML = md
        ? renderMarkdownPreview(md)
        : '<span style="color:var(--text-faint);font-style:italic">No description yet</span>';
    });
  }

  // Save
  content.querySelector('#save-btn')?.addEventListener('click', async () => {
    const btn = content.querySelector('#save-btn') as HTMLButtonElement;
    btn.disabled = true;
    btn.textContent = 'Saving...';

    const updates: Record<string, unknown> = {};

    const displayName = (content.querySelector('#edit-display-name') as HTMLInputElement).value.trim();
    const tagline = (content.querySelector('#edit-tagline') as HTMLInputElement).value.trim();
    const description = (content.querySelector('#edit-description') as HTMLTextAreaElement).value.trim();
    const category = (content.querySelector('#edit-category') as HTMLSelectElement).value;
    const license = (content.querySelector('#edit-license') as HTMLInputElement).value.trim();
    const repoUrl = (content.querySelector('#edit-repo-url') as HTMLInputElement).value.trim();
    const homepageUrl = (content.querySelector('#edit-homepage-url') as HTMLInputElement).value.trim();

    if (!displayName) {
        showToast('Display name is required', 'error');
        btn.disabled = false;
        btn.textContent = 'Save Changes';
        return;
    }
    updates.display_name = displayName;
    updates.tagline = tagline || null;
    updates.description = description;
    updates.category = category;
    updates.license = license || null;
    updates.repo_url = repoUrl || null;
    updates.homepage_url = homepageUrl || null;

    // Compatibility
    updates.compat_claude_code = (content.querySelector('#compat-claude-code') as HTMLInputElement).checked ? 1 : 0;
    updates.compat_cursor = (content.querySelector('#compat-cursor') as HTMLInputElement).checked ? 1 : 0;
    updates.compat_codex = (content.querySelector('#compat-codex') as HTMLInputElement).checked ? 1 : 0;
    updates.compat_copilot = (content.querySelector('#compat-copilot') as HTMLInputElement).checked ? 1 : 0;
    updates.compat_gemini = (content.querySelector('#compat-gemini') as HTMLInputElement).checked ? 1 : 0;

    try {
      await updatePackage(pkg.author_id, pkg.name, updates);
      showToast('Package updated', 'success');
      // Clear cached package so detail reloads fresh data
      setState({ selectedPackage: null, selectedAuthor: pkg.author_id, selectedName: pkg.name });
      navigate('detail');
    } catch (err: any) {
      showToast(`Update failed: ${err}`, 'error');
      btn.disabled = false;
      btn.textContent = 'Save Changes';
    }
  });
}

function renderMarkdownPreview(text: string): string {
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
      i++;
      html.push(`<pre style="background:var(--bg-tertiary);padding:12px;border-radius:6px;overflow-x:auto;font-family:'Geist Mono',monospace;font-size:13px;color:var(--text-primary);margin:8px 0;line-height:1.5"><code>${esc(codeLines.join('\n'))}</code></pre>`);
      continue;
    }

    if (line.trim() === '') { i++; continue; }

    // Headings
    const hm = line.match(/^(#{1,6})\s+(.+)$/);
    if (hm) {
      const sz: Record<number, string> = { 1: '22px', 2: '18px', 3: '16px', 4: '14px', 5: '13px', 6: '12px' };
      html.push(`<div style="font-size:${sz[hm[1].length] || '14px'};font-weight:600;color:var(--text-primary);margin:16px 0 8px">${inlineMd(hm[2])}</div>`);
      i++; continue;
    }

    // Unordered list
    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*[-*+]\s+/, ''));
        i++;
      }
      html.push(`<ul style="margin:6px 0;padding-left:20px">${items.map(li => `<li style="margin:3px 0;line-height:1.5">${inlineMd(li)}</li>`).join('')}</ul>`);
      continue;
    }

    // Ordered list
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*\d+[.)]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*\d+[.)]\s+/, ''));
        i++;
      }
      html.push(`<ol style="margin:6px 0;padding-left:20px">${items.map(li => `<li style="margin:3px 0;line-height:1.5">${inlineMd(li)}</li>`).join('')}</ol>`);
      continue;
    }

    // Blockquote
    if (/^>\s?/.test(line)) {
      const qlines: string[] = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        qlines.push(lines[i].replace(/^>\s?/, ''));
        i++;
      }
      html.push(`<blockquote style="margin:8px 0;padding:6px 14px;border-left:3px solid var(--accent);color:var(--text-secondary);font-style:italic">${qlines.map(l => inlineMd(l)).join('<br>')}</blockquote>`);
      continue;
    }

    // Paragraph
    const plines: string[] = [];
    while (i < lines.length && lines[i].trim() !== '' && !lines[i].trimStart().startsWith('```') && !/^#{1,6}\s+/.test(lines[i]) && !/^\s*[-*+]\s+/.test(lines[i]) && !/^\s*\d+[.)]\s+/.test(lines[i]) && !/^>\s?/.test(lines[i])) {
      plines.push(lines[i]);
      i++;
    }
    if (plines.length > 0) {
      html.push(`<p style="margin:6px 0;line-height:1.6">${inlineMd(plines.join('\n').replace(/\n/g, '<br>'))}</p>`);
    }
  }

  return html.join('');
}

function inlineMd(text: string): string {
  // Escape HTML first to prevent XSS injection
  let safe = esc(text);

  // Images: only allow http/https src
  safe = safe.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_m, alt, url) => {
    if (!/^https?:\/\//i.test(url)) return esc(alt);
    return `<img src="${esc(url)}" alt="${esc(alt)}" style="max-width:100%;height:auto">`;
  });
  // Links: only allow http/https href
  safe = safe.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_m, label, url) => {
    if (!/^https?:\/\//i.test(url)) return label;
    return `<a href="${esc(url)}" target="_blank" rel="noopener" style="color:var(--accent)">${label}</a>`;
  });
  // Bold
  safe = safe.replace(/\*\*(.+?)\*\*/g, '<strong style="color:var(--text-primary)">$1</strong>');
  // Italic
  safe = safe.replace(/\*(.+?)\*/g, '<em>$1</em>');
  // Strikethrough
  safe = safe.replace(/~~(.+?)~~/g, '<del>$1</del>');
  // Inline code
  safe = safe.replace(/`([^`]+)`/g, '<code style="background:var(--bg-tertiary);padding:1px 5px;border-radius:3px;font-family:\'Geist Mono\',monospace;font-size:0.9em">$1</code>');
  // Auto-link bare URLs — skip URLs already inside href="..." or src="..."
  safe = safe.replace(/(^|[\s(])(?![^<]*>)((https?:\/\/)[^\s<)]+)/g, (_m, prefix, url) => {
    return `${prefix}<a href="${esc(url)}" target="_blank" rel="noopener" style="color:var(--accent)">${url}</a>`;
  });

  return safe;
}
