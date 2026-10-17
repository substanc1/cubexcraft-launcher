<script>
  import './login.css';
  // scenes/login/login.fxml + methods/loginpassword.fxml + LoginScene/AuthFlow/LoginAndPasswordAuthMethod/AuthButton
  import { app, t, processRequest, confirmTwoFa, switchScene, errorHandle, saveSettings, startDrag } from '../lib/app.svelte.js';
  import { call } from '../lib/backend.js';
  import { onMount } from 'svelte';
  import IconButton from '../components/IconButton.svelte';
  import Checkbox from '../components/Checkbox.svelte';
  import CutButton from '../components/CutButton.svelte';
  import { tooltip } from '../lib/tooltip.js';
  import logo from '../assets/images/logo.png';

  const CREATE_ACCOUNT_URL = 'https://cubexcraft.site/register';
  const FORGOT_PASS_URL = 'https://cubexcraft.site/forgot-password';

  const s = app.settings || {};
  let login = $state(s.login || '');
  let password = $state('');
  let reveal = $state(false);
  let hasSaved = $state(!!(s.login && s.hasSavedSession));
  let autoenter = $state(!!s.autoAuth);
  let savePassword = $state(!!s.login || !!s.hasSavedSession);
  let validation = $state(null);
  let btn = $state(s.login ? 'active' : 'unactive');
  let busy = false;
  const reduce = !!s.reduceEffects;

  function onLoginInput() {
    btn = login === '' ? 'unactive' : 'active';
  }

  async function onSuccessLogin(user, recentLogin, fromSaved) {
    app.user = user;
    if (savePassword) {
      app.settings.login = recentLogin;
      await call('auth_remember', { remember: true }).catch(() => {});
    } else {
      app.settings.login = null;
      await call('auth_remember', { remember: false }).catch(() => {});
    }
    app.settings.autoAuth = autoenter;
    saveSettings();
    const profiles = await processRequest(t('overlay.processing.text.profiles'), call('profiles_list'));
    app.profiles = profiles;
    await switchScene('servermenu');
    call('timing_mark', { what: 'меню серверов показано' }).catch(() => {});
  }

  // AuthFlow.loginWithGui -> LoginAndPasswordAuthMethod.onAuthClicked -> AuthFlow.login
  async function loginWithGui() {
    if (busy) return;
    btn = 'unactive';
    const savedOnly = password === '' && hasSaved;
    if (!login.trim() || (password === '' && !savedOnly)) {
      validation = t('scenes.login.fillFields');
      return;
    }
    validation = null;
    busy = true;
    try {
      const res = await processRequest(
        t('overlay.processing.text.auth'),
        savedOnly ? call('auth_restore', { login }) : call('auth_login', { login, password }),
        { onError: () => {} },
      );
      // включено подтверждение входа в Telegram - ждём «✅ Это я» в боте
      const user = res && res.twofa ? await confirmTwoFa(res.twofa) : res;
      await onSuccessLogin(user, login, savedOnly);
    } catch (e) {
      if (e === '2fa.cancelled') {
        btn = 'active';
        return;
      }
      errorHandle(e);
      btn = 'error';
      if (savedOnly) hasSaved = false; // сохранённая сессия умерла - просим пароль
    } finally {
      busy = false;
    }
  }

  function onSavePassword(on) {
    if (!on) {
      app.settings.login = null;
      call('auth_remember', { remember: false }).catch(() => {});
      saveSettings();
    }
  }
  function onAutoenter(on) {
    app.settings.autoAuth = on;
    saveSettings();
  }

  onMount(async () => {
    // LoginScene.runAutoAuth: автовход сохранённой сессией
    if (s.autoAuth && hasSaved) {
      busy = true;
      try {
        const user = await processRequest(t('overlay.processing.text.auth'), call('auth_restore', { login }), { onError: () => {} });
        busy = false;
        await onSuccessLogin(user, login, true);
      } catch (e) {
        busy = false;
        hasSaved = false;
      }
    }
  });

  function keydown(e) {
    if (e.key === 'Enter') loginWithGui();
  }
</script>

<div class="lg-layout bg-image" class:no-effects={reduce} onmousedown={startDrag} role="presentation">
  <div class="lg-header">
    <div class="lg-brand">
      <img src={logo} alt="" />
      <span class="lg-word"><span>CUBE</span><span class="lg-accent">X</span><span>CRAFT</span></span>
    </div>
    <div class="lg-controls">
      <button type="button" class="lg-statusPill" use:tooltip={t('scenes.login.serverStatusTooltip')} data-no-drag>
        <span class="lg-dot">●</span><span class="lg-statusText">{t('scenes.login.serverStatusOnline')}</span>
      </button>
      <IconButton glyph="⚙" tip={t('scenes.serverinfo.settingsTooltip')} onclick={() => switchScene('settings')} />
      <IconButton glyph="—" tip={t('scenes.servermenu.minimizeTooltip')} onclick={() => call('window_minimize')} />
      <IconButton glyph="✕" kind="exit" tip={t('scenes.servermenu.exitTooltip')} onclick={() => call('window_close')} />
    </div>
  </div>

  <div class="lg-center">
    <div class="lg-column">
      <div class="lg-secure">
        <span class="lg-secureLink">{t('scenes.login.secureLink')}</span>
        <span class="lg-divider"></span>
      </div>
      <div class="lg-headingGroup" class:lg-flicker={!reduce}>
        <span class="lg-heading lg-ghostYellow">{t('scenes.login.heading')}</span>
        <span class="lg-heading lg-ghostRed">{t('scenes.login.heading')}</span>
        <span class="lg-heading lg-main">{t('scenes.login.heading')}</span>
      </div>
      <div class="lg-subheading">{t('scenes.login.subheading')}</div>

      <div class="lg-content">
        <label class="lg-field lg-login">
          <input type="text" bind:value={login} oninput={onLoginInput} onkeydown={keydown}
                 placeholder={t('scenes.login.login')} spellcheck="false" autocomplete="off" />
        </label>
        <label class="lg-field lg-password">
          <input type={reveal ? 'text' : 'password'} bind:value={password} onkeydown={keydown}
                 placeholder={hasSaved ? t('scenes.login.password.saved') : t('scenes.login.password')}
                 spellcheck="false" autocomplete="off" />
          <button type="button" class="lg-revealToggle" class:selected={reveal} onclick={() => (reveal = !reveal)} data-no-drag>
            {reveal ? t('scenes.login.hidePassword') : t('scenes.login.showPassword')}
          </button>
        </label>
      </div>

      <div class="lg-checks">
        <Checkbox bind:checked={autoenter} label={t('scenes.login.autoenter')} onchange={onAutoenter} />
        <Checkbox bind:checked={savePassword} label={t('scenes.login.savePassword')} onchange={onSavePassword} />
        <span class="lg-grow"></span>
        <button type="button" class="lg-forgotPass" onclick={() => call('open_url', { url: FORGOT_PASS_URL })}>{t('scenes.login.forgotPass')}</button>
      </div>

      {#if validation}
        <div class="lg-validationBanner"><span>!</span><span>{validation}</span></div>
      {/if}

      <CutButton variant={btn} class="authButton" onclick={loginWithGui}>
        {btn === 'error' ? 'ERROR' : t('scenes.login.goAuth')}
      </CutButton>

      <div class="lg-registerPane">
        <span class="lg-textLogin">{t('scenes.login.textLogin')}</span><button type="button" class="lg-createAccount"
          onclick={() => call('open_url', { url: CREATE_ACCOUNT_URL })}>{t('scenes.login.createAccount')}</button>
      </div>

      <button type="button" class="lg-secondaryButton">{t('scenes.login.discordChannel')}</button>
    </div>
  </div>

  {#if !reduce}
    <div class="lg-scanlines"></div>
    <div class="lg-sweepBand"></div>
  {/if}

  <div class="lg-footer">
    <div class="lg-net"><span class="lg-dot">●</span><span>{t('scenes.login.footerNet')}</span></div>
    <span class="lg-copyright">{t('scenes.login.footerCopyright')}</span>
  </div>
</div>
