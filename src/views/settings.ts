import { getState, setState } from '../lib/state';
import { setAuthToken, clearAuthToken, getAuthStatus } from '../lib/api';
import { showToast } from '../components/toast';
import { renderSidebar } from '../components/sidebar';

export async function renderSettings() {
  const content = document.getElementById('content');
  if (!content) return;

  const state = getState();
  const authenticated = state.authenticated;

  const authSection = authenticated
    ? `
      <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px;padding:14px 16px;background:rgba(34,197,94,0.08);border:1px solid rgba(34,197,94,0.2);border-radius:8px">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
        <div>
          <div style="font-size:14px;color:#22c55e;font-weight:500">Connected to SkillVault</div>
          <div style="font-size:12px;color:var(--text-muted);margin-top:2px">Your API token is securely stored in the system keychain.</div>
        </div>
      </div>
      <button class="btn btn--sm" id="sign-out-btn" style="color:var(--error);border-color:rgba(239,68,68,0.3)">Disconnect</button>
    `
    : `
      <p style="font-size:13px;color:var(--text-secondary);margin-bottom:20px">
        Connect your SkillVault account to install paid packages, star skills, publish, and leave reviews.
      </p>

      <div style="display:flex;flex-direction:column;gap:20px;max-width:480px">
        <!-- Step 1 -->
        <div style="display:flex;gap:12px;align-items:flex-start">
          <div style="width:24px;height:24px;border-radius:50%;background:var(--accent);color:#fff;display:flex;align-items:center;justify-content:center;font-family:'Geist Mono',monospace;font-size:12px;font-weight:600;flex-shrink:0">1</div>
          <div style="flex:1">
            <div style="font-size:14px;color:var(--text-primary);font-weight:500;margin-bottom:4px">Sign in on SkillVault</div>
            <p style="font-size:12px;color:var(--text-muted);margin-bottom:8px">Opens skillvault.md in your browser. Sign in with GitHub.</p>
            <button class="btn btn--sm" id="open-website-btn" style="display:inline-flex;align-items:center;gap:6px">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
              Open SkillVault
            </button>
          </div>
        </div>

        <!-- Step 2 -->
        <div style="display:flex;gap:12px;align-items:flex-start">
          <div style="width:24px;height:24px;border-radius:50%;background:var(--accent);color:#fff;display:flex;align-items:center;justify-content:center;font-family:'Geist Mono',monospace;font-size:12px;font-weight:600;flex-shrink:0">2</div>
          <div style="flex:1">
            <div style="font-size:14px;color:var(--text-primary);font-weight:500;margin-bottom:4px">Create an API token</div>
            <p style="font-size:12px;color:var(--text-muted);margin-bottom:8px">Go to your dashboard and click "Create Token". Copy the <span style="font-family:'Geist Mono',monospace;color:var(--text-secondary)">svt_...</span> token.</p>
            <button class="btn btn--sm" id="open-tokens-btn">Open Token Page</button>
          </div>
        </div>

        <!-- Step 3 -->
        <div style="display:flex;gap:12px;align-items:flex-start">
          <div style="width:24px;height:24px;border-radius:50%;background:var(--accent);color:#fff;display:flex;align-items:center;justify-content:center;font-family:'Geist Mono',monospace;font-size:12px;font-weight:600;flex-shrink:0">3</div>
          <div style="flex:1">
            <div style="font-size:14px;color:var(--text-primary);font-weight:500;margin-bottom:8px">Paste your token here</div>
            <div class="settings-row">
              <div style="position:relative;flex:1;display:flex">
                <input class="settings-input" id="token-input" type="password" placeholder="svt_..." value="" style="flex:1;padding-right:40px">
                <button id="toggle-token-btn" style="position:absolute;right:8px;top:50%;transform:translateY(-50%);background:none;border:none;cursor:pointer;color:var(--text-muted);padding:4px;display:flex;align-items:center" title="Show/hide">
                  <svg id="eye-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                </button>
              </div>
              <button class="btn btn--primary btn--sm" id="save-token-btn">Connect</button>
            </div>
            <p style="font-size:11px;color:var(--text-faint);margin-top:6px">Stored securely in your system keychain. Never sent anywhere except skillvault.md.</p>
          </div>
        </div>
      </div>
    `;

  content.innerHTML = `
    <div class="view-header">
      <div class="view-header-title">
        <h1 class="h1">Settings</h1>
      </div>
    </div>

    <div class="settings-section">
      <h2 class="h2" style="margin-bottom:16px">Account</h2>
      ${authSection}
    </div>

    <div class="settings-section">
      <h2 class="h2" style="margin-bottom:16px">Keyboard Shortcuts</h2>
      <div style="display:grid;grid-template-columns:1fr auto;gap:6px 32px;font-size:13px;max-width:440px">
        <div style="color:var(--text-faint);font-family:'Geist Mono',monospace;font-size:10px;letter-spacing:0.5px;grid-column:1/-1;margin:4px 0 2px">NAVIGATION</div>
        <span style="color:var(--text-secondary)">Go back</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; [</span>
        <span style="color:var(--text-secondary)">Go forward</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; ]</span>
        <span style="color:var(--text-secondary)">Escape / go back</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">Esc</span>
        <div style="color:var(--text-faint);font-family:'Geist Mono',monospace;font-size:10px;letter-spacing:0.5px;grid-column:1/-1;margin:8px 0 2px">VIEWS</div>
        <span style="color:var(--text-secondary)">My Skills</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 1</span>
        <span style="color:var(--text-secondary)">Publish</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 2</span>
        <span style="color:var(--text-secondary)">Browse</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 3</span>
        <span style="color:var(--text-secondary)">New</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 4</span>
        <span style="color:var(--text-secondary)">Trending</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 5</span>
        <span style="color:var(--text-secondary)">Plugins</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 6</span>
        <span style="color:var(--text-secondary)">Settings</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; 7 &nbsp;or&nbsp; &#8984; ,</span>
        <div style="color:var(--text-faint);font-family:'Geist Mono',monospace;font-size:10px;letter-spacing:0.5px;grid-column:1/-1;margin:8px 0 2px">ACTIONS</div>
        <span style="color:var(--text-secondary)">Search / focus search bar</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; F</span>
        <span style="color:var(--text-secondary)">Refresh / Scan</span>
        <span style="font-family:'Geist Mono',monospace;font-size:12px;color:var(--text-muted)">&#8984; R</span>
      </div>
    </div>

    <div class="settings-section">
      <h2 class="h2" style="margin-bottom:16px">About</h2>
      <div style="font-size:13px;color:var(--text-secondary);line-height:22px">
        <p><strong style="color:var(--text-primary)">SkillVault Desktop</strong> v0.1.0</p>
        <p style="margin-top:4px">The mod manager for AI coding skills.</p>
        <p style="margin-top:8px;color:var(--text-muted)">
          Open source &mdash; MIT License
        </p>
      </div>
    </div>
  `;

  const openInBrowser = async (url: string) => {
    try {
      const { open } = window.__TAURI__.shell;
      await open(url);
    } catch {
      // Ignore
    }
  };

  if (authenticated) {
    content.querySelector('#sign-out-btn')?.addEventListener('click', async () => {
      try {
        await clearAuthToken();
        setState({ authenticated: false });
        showToast('Disconnected', 'success');
        renderSidebar();
        renderSettings();
      } catch (e: any) {
        showToast(`Failed: ${e}`, 'error');
      }
    });
  } else {
    content.querySelector('#open-website-btn')?.addEventListener('click', () => {
      openInBrowser('https://skillvault.md/sign-in');
    });

    content.querySelector('#open-tokens-btn')?.addEventListener('click', () => {
      openInBrowser('https://skillvault.md/dashboard');
    });

    content.querySelector('#toggle-token-btn')?.addEventListener('click', () => {
      const input = content.querySelector('#token-input') as HTMLInputElement;
      if (!input) return;
      input.type = input.type === 'password' ? 'text' : 'password';
    });

    content.querySelector('#save-token-btn')?.addEventListener('click', async () => {
      const input = content.querySelector('#token-input') as HTMLInputElement;
      const token = input.value.trim();
      if (!token) {
        showToast('Please enter a token', 'error');
        return;
      }

      try {
        await setAuthToken(token);
        setState({ authenticated: true });
        showToast('Connected to SkillVault', 'success');
        renderSidebar();
        renderSettings();
      } catch (e: any) {
        showToast(`Failed: ${e}`, 'error');
      }
    });
  }
}
