<iframe
    style="width:100%; height:2200px; border:none;"
    srcdoc='

<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  .wrap { padding: 1rem 0 2rem; }
  .col-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
  .panel {
    border: 0.5px solid var(--color-border-tertiary);
    border-radius: var(--border-radius-lg);
    overflow: hidden;
  }
  .panel-header {
    padding: 10px 14px;
    display: flex; align-items: center; gap: 8px;
    border-bottom: 0.5px solid var(--color-border-tertiary);
    background: var(--color-background-secondary);
  }
  .panel-header .title { font-size: 13px; font-weight: 500; color: var(--color-text-primary); }
  .panel-header .sub   { font-size: 11px; color: var(--color-text-tertiary); margin-left: auto; }
  .panel-body { padding: 10px 12px; }

  .tree { font-family: var(--font-mono); font-size: 11.5px; line-height: 2; }
  .row  { display: flex; align-items: center; gap: 4px; padding: 0 2px; border-radius: 3px; }
  .row:hover { background: var(--color-background-secondary); }
  .con  { color: var(--color-text-tertiary); user-select: none; flex-shrink: 0; font-size: 11px; }
  .sp   { display: inline-block; width: 16px; flex-shrink: 0; }
  .sp2  { display: inline-block; width: 32px; flex-shrink: 0; }
  .sp3  { display: inline-block; width: 48px; flex-shrink: 0; }
  .dir  { color: #185FA5; font-weight: 500; }
  .file { color: var(--color-text-primary); }
  .ann  { font-family: var(--font-sans); font-size: 10px; color: var(--color-text-tertiary); margin-left: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .badge {
    font-family: var(--font-sans); font-size: 9.5px; font-weight: 500;
    padding: 1px 5px; border-radius: 20px; margin-left: 3px; white-space: nowrap; flex-shrink: 0;
  }
  .b-entry  { background: #E6F1FB; color: #0C447C; }
  .b-data   { background: #E1F5EE; color: #085041; }
  .b-logic  { background: #FAEEDA; color: #633806; }
  .b-async  { background: #EEEDFE; color: #3C3489; }
  .b-api    { background: #FAECE7; color: #712B13; }
  .b-auth   { background: #FBEAF0; color: #72243E; }
  .b-ch     { background: #EAF3DE; color: #27500A; }

  .section-gap { height: 8px; }

  .full-panel {
    border: 0.5px solid var(--color-border-tertiary);
    border-radius: var(--border-radius-lg);
    overflow: hidden;
    margin-bottom: 1rem;
  }

  .channel-row {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 14px;
    border-bottom: 0.5px solid var(--color-border-tertiary);
    font-size: 12px;
  }
  .channel-row:last-child { border-bottom: none; }
  .ch-name { font-family: var(--font-mono); font-size: 11px; color: var(--color-text-primary); width: 160px; flex-shrink: 0; }
  .ch-dir  { font-size: 11px; color: var(--color-text-tertiary); width: 220px; flex-shrink: 0; }
  .ch-desc { font-size: 11px; color: var(--color-text-secondary); }

  .legend {
    display: flex; flex-wrap: wrap; gap: 6px;
    padding: 10px 14px;
    border-top: 0.5px solid var(--color-border-tertiary);
    background: var(--color-background-secondary);
  }
  .legend-item { display: flex; align-items: center; gap: 4px; font-family: var(--font-sans); font-size: 11px; color: var(--color-text-secondary); }
</style>

<div class="wrap">

  <!-- top: two columns -->
  <div class="col-grid">

    <!-- LEFT: Custom Phantom -->
    <div class="panel">
      <div class="panel-header">
        <i class="ti ti-wallet" aria-hidden="true" style="color:#185FA5;font-size:15px"></i>
        <span class="title">Custom Phantom</span>
        <span class="sub">identity + keys</span>
      </div>
      <div class="panel-body">
        <div class="tree">

          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">src/phantom/</span></div>

          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>

          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">keypair.rs</span><span class="badge b-logic">logic</span><span class="ann">generate ed25519 keypair, BIP39</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">recovery.rs</span><span class="badge b-logic">logic</span><span class="ann">restore wallet from phrase/privkey</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">encrypt.rs</span><span class="badge b-logic">logic</span><span class="ann">AES-256-GCM encrypt/decrypt privkey</span></div>

          <div class="section-gap"></div>

          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">db/</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">user.rs</span><span class="badge b-data">data</span><span class="ann">User, WalletEntry structs</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">store.rs</span><span class="badge b-async">state</span><span class="ann">UserDb — Arc&lt;RwLock&lt;HashMap&gt;&gt;</span></div>

          <div class="section-gap"></div>

          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">auth/</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">password.rs</span><span class="badge b-auth">auth</span><span class="ann">argon2 hash/verify</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">jwt.rs</span><span class="badge b-auth">auth</span><span class="ann">issue/verify access+refresh tokens</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">google.rs</span><span class="badge b-auth">auth</span><span class="ann">OAuth2 flow (optional)</span></div>

        </div>
      </div>
    </div>

    <!-- RIGHT: Mini Solana Runtime -->
    <div class="panel">
      <div class="panel-header">
        <i class="ti ti-cpu" aria-hidden="true" style="color:#854F0B;font-size:15px"></i>
        <span class="title">Mini Solana Runtime</span>
        <span class="sub">execution + consensus</span>
      </div>
      <div class="panel-body">
        <div class="tree">

          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">src/runtime/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">simulator.rs</span><span class="badge b-async">task</span><span class="ann">orchestrator — wires all tasks + query loop</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">poh.rs</span><span class="badge b-logic">logic</span><span class="ann">SHA-256 hash chain</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">mempool.rs</span><span class="badge b-async">state</span><span class="ann">Arc&lt;Mutex&lt;VecDeque&gt;&gt;</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">network.rs</span><span class="badge b-async">state</span><span class="ann">NetworkBus — broadcast channel</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">scheduler.rs</span><span class="badge b-logic">logic</span><span class="ann">stake-weighted leader schedule</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">consensus.rs</span><span class="badge b-async">task</span><span class="ann">2/3 vote threshold + finalization</span></div>

          <div class="section-gap"></div>

          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">accounts/</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">db.rs</span><span class="badge b-async">state</span><span class="ann">AccountsDb — balances, snapshot, rollback</span></div>

          <div class="section-gap"></div>

          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">executor/</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">executor.rs</span><span class="badge b-logic">logic</span><span class="ann">verify sig, lock, snapshot, execute</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">builder.rs</span><span class="badge b-logic">logic</span><span class="ann">assemble Block from executed txs</span></div>

          <div class="section-gap"></div>

          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">validator/</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span><span class="badge b-async">task</span><span class="ann">Validator — select! event loop</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">state.rs</span><span class="badge b-data">data</span><span class="ann">ValidatorState enum</span></div>

        </div>
      </div>
    </div>

  </div>

  <div style="height:1rem"></div>

  <!-- FULL WIDTH: API layer -->
  <div class="full-panel">
    <div class="panel-header">
      <i class="ti ti-api" aria-hidden="true" style="color:#993C1D;font-size:15px"></i>
      <span class="title">API Layer — src/api/</span>
      <span class="sub">single Axum router, two handler groups</span>
    </div>
    <div class="panel-body">
      <div class="col-grid">

        <div class="tree">
          <div class="row"><span class="file">mod.rs</span></div>
          <div class="row"><span class="file">router.rs</span><span class="badge b-api">Axum</span><span class="ann">builds full router, mounts both groups</span></div>
          <div class="row"><span class="file">state.rs</span><span class="badge b-api">Axum</span><span class="ann">AppState — channels only, no Arc state</span></div>
          <div class="row"><span class="file">ws.rs</span><span class="badge b-api">Axum</span><span class="ann">WebSocket — subscribes to event_tx</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">middleware/</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">auth.rs</span><span class="badge b-auth">auth</span><span class="ann">JWT extractor — protects phantom routes</span></div>
        </div>

        <div class="tree">
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">handlers/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">mod.rs</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">phantom/</span><span class="ann">JWT protected</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">register.rs</span><span class="ann">POST /auth/register</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">login.rs</span><span class="ann">POST /auth/login</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">wallets.rs</span><span class="ann">GET  /user/wallets</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">wallet_new.rs</span><span class="ann">POST /user/wallet/new</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">wallet_import.rs</span><span class="ann">POST /user/wallet/import</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">recovery.rs</span><span class="ann">GET  /user/recovery-phrase</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">privkey.rs</span><span class="ann">GET  /user/private-key</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">simulation/</span><span class="ann">open read, JWT for write</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">tx.rs</span><span class="ann">POST /tx</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">airdrop.rs</span><span class="ann">POST /airdrop</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">balances.rs</span><span class="ann">GET  /balances</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">chain.rs</span><span class="ann">GET  /chain</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">├──</span><span class="file">wallet_info.rs</span><span class="ann">GET  /wallet/:address</span></div>
          <div class="row"><span class="sp"></span><span class="sp"></span><span class="con">└──</span><span class="file">validators.rs</span><span class="ann">GET  /validators</span></div>
        </div>

      </div>
    </div>
  </div>

  <!-- channels -->
  <div class="full-panel">
    <div class="panel-header">
      <i class="ti ti-arrows-exchange" aria-hidden="true" style="color:#0F6E56;font-size:15px"></i>
      <span class="title">Tokio channels between them</span>
    </div>
    <div class="channel-row">
      <span class="ch-name">runtime_query_tx</span>
      <span class="ch-dir">Phantom ──────────────► Runtime</span>
      <span class="ch-desc">read balance, tx history, chain — oneshot response</span>
    </div>
    <div class="channel-row">
      <span class="ch-name">mempool_tx</span>
      <span class="ch-dir">Phantom ──────────────► Runtime</span>
      <span class="ch-desc">submit signed tx or airdrop tx</span>
    </div>
    <div class="channel-row">
      <span class="ch-name">event_tx</span>
      <span class="ch-dir">Runtime ──────────────► Phantom + WS</span>
      <span class="ch-desc">slot ticks, block finalized, balances updated, tx results</span>
    </div>
  </div>

  <!-- shared root files -->
  <div class="full-panel">
    <div class="panel-header">
      <i class="ti ti-file-code" aria-hidden="true" style="color:var(--color-text-secondary);font-size:15px"></i>
      <span class="title">Root src/ files</span>
    </div>
    <div class="panel-body">
      <div class="tree" style="display:grid;grid-template-columns:1fr 1fr;gap:0 2rem">
        <div class="row"><span class="file">main.rs</span><span class="badge b-entry">entry</span><span class="ann">spawn runtime + start Axum</span></div>
        <div class="row"><span class="file">config.rs</span><span class="badge b-data">data</span><span class="ann">SimConfig, ValidatorConfig</span></div>
        <div class="row"><span class="file">types.rs</span><span class="badge b-data">data</span><span class="ann">Transaction, Block, Vote, RuntimeQuery, SimEvent</span></div>
        <div class="row"><span class="file">error.rs</span><span class="badge b-data">data</span><span class="ann">ApiError, RuntimeError, MempoolError</span></div>
      </div>
    </div>
    <div class="legend">
      <div class="legend-item"><span class="badge b-entry">entry</span>startup</div>
      <div class="legend-item"><span class="badge b-data">data</span>structs only</div>
      <div class="legend-item"><span class="badge b-logic">logic</span>pure logic</div>
      <div class="legend-item"><span class="badge b-async">task/state</span>tokio async</div>
      <div class="legend-item"><span class="badge b-api">Axum</span>HTTP layer</div>
      <div class="legend-item"><span class="badge b-auth">auth</span>JWT / crypto</div>
    </div>
  </div>

  <!-- frontend -->
  <div class="full-panel">
    <div class="panel-header">
      <i class="ti ti-brand-react" aria-hidden="true" style="color:#534AB7;font-size:15px"></i>
      <span class="title">Frontend — frontend/src/</span>
      <span class="sub">Bun + React + TypeScript</span>
    </div>
    <div class="panel-body">
      <div class="col-grid">
        <div class="tree">
          <div class="row"><span class="file">main.tsx</span><span class="ann">ReactDOM entry</span></div>
          <div class="row"><span class="file">App.tsx</span><span class="ann">root layout</span></div>
          <div class="row"><span class="file">vite.config.ts</span><span class="ann">proxy → :3001</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">types/</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">sim.ts</span><span class="ann">mirrors all Rust types</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">store/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">simulation.ts</span><span class="ann">zustand — SimState</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">wallet.ts</span><span class="ann">zustand — current user + wallets</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">hooks/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">useWebSocket.ts</span><span class="ann">connects to /ws, feeds store</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">useWalletInfo.ts</span><span class="ann">react-query GET /wallet/:address</span></div>
        </div>
        <div class="tree">
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">lib/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">wallet.ts</span><span class="ann">createWallet, signTx — @noble/ed25519</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">api.ts</span><span class="ann">typed fetch wrappers for all routes</span></div>
          <div class="section-gap"></div>
          <div class="row"><span class="icon ti ti-folder" aria-hidden="true" style="color:#185FA5;font-size:12px"></span><span class="dir">components/</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">SlotHeader.tsx</span><span class="ann">slot · epoch · leader</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">ValidatorCard.tsx</span><span class="ann">stake · state · last vote</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">BalanceTable.tsx</span><span class="ann">live balance + delta animation</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">BlockFeed.tsx</span><span class="ann">finalized block list</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">EventLog.tsx</span><span class="ann">terminal stream</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">TransactionForm.tsx</span><span class="ann">sign + POST /tx</span></div>
          <div class="row"><span class="sp"></span><span class="con">├──</span><span class="file">Onboarding.tsx</span><span class="ann">register · login · create wallet</span></div>
          <div class="row"><span class="sp"></span><span class="con">└──</span><span class="file">WalletPanel.tsx</span><span class="ann">address · balance · tx history</span></div>
        </div>
      </div>
    </div>
  </div>

</div>'
></iframe>