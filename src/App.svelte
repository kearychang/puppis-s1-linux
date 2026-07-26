<script lang="ts">
  import { onMount } from "svelte";
  import packageMetadata from "../package.json";
  import type { ApplicationSnapshot, DeviceRole, DiagnosticsBundle, OperationFailure, RadioBand, RadioSnapshot, RadioUpdate, StatusDimension } from "./lib/contracts";
  import { uiText } from "./lib/strings";

  export let snapshot: ApplicationSnapshot;
  export let onSelectCandidate: (candidateId: string) => void | ApplicationSnapshot | Promise<void | ApplicationSnapshot> = () => {};
  export let onVerify: () => void | ApplicationSnapshot | Promise<void | ApplicationSnapshot> = () => {};
  export let onPreviewDiagnostics: () => Promise<DiagnosticsBundle> = async () => ({ preview: "", suggestedName: "puppis-s1-diagnostics.json" });
  export let onExportDiagnostics: (path: string, approvedPreview: string) => void | Promise<void> = () => {};
  export let onInspectNetwork: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onEnableManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onDisableManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onRemoveManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onReadRadios: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onApplyRadio: (band: RadioBand, update: RadioUpdate) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onSetDeviceRole: (role: DeviceRole, confirmed: boolean) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onRefreshClientEvidence: (telemetryVisible: boolean) => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onRefreshStatus: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onSaveClientLabel: (hardwareAddress: string, label: string) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onRenameSavedClient: (hardwareAddress: string, label: string) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onForgetSavedClient: (hardwareAddress: string) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onClearSavedClients: (confirmed: boolean) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onResetSavedClients: (confirmed: boolean) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onReassociateSavedClient: (previousHardwareAddress: string, newHardwareAddress: string, confirmed: boolean) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onAcceptRecoveryBaseline: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let subscribeSnapshots: (handler: (next: ApplicationSnapshot) => void) => Promise<() => void> = async () => () => {};
  export let loadSnapshot: (() => Promise<ApplicationSnapshot>) | null = null;
  type Section = "overview" | "network" | "wifi" | "diagnostics" | "about";
  let activeSection: Section = "overview";
  let textScale: 100 | 125 | 150 = 100;
  let diagnostics: DiagnosticsBundle | null = null;
  let exportPath = "puppis-s1-diagnostics.json";
  let ssidDrafts: Record<string, string> = {};
  let channelDrafts: Record<string, string> = {};
  let fiveGhzPassword = "";
  let transitionTarget: DeviceRole | null = null;
  let confirmManagedReplacement = false;
  let failure: OperationFailure | null = null;
  let operationPending = false;
  let pendingLabel = "";
  let clientLabelDrafts: Record<string, string> = {};
  let savedLabelDrafts: Record<string, string> = {};
  let refreshingStatus = false;
  let nowSeconds = Math.floor(Date.now() / 1000);
  let confirmClearSaved = false;
  let confirmResetSaved = false;
  let reassociation: { previousHardwareAddress: string; newHardwareAddress: string } | null = null;
  let unobservedSavedClients: ApplicationSnapshot["savedClients"] = [];
  $: unobservedSavedClients = snapshot.savedClients
    .filter((saved) => !snapshot.clientEvidence.some((observed) => observed.hardwareAddress === saved.hardwareAddress))
    .sort((left, right) => right.lastObservedAt - left.lastObservedAt);
  $: for (const radio of snapshot.radios) {
    if (ssidDrafts[radio.band] === undefined) ssidDrafts[radio.band] = radio.ssid;
    if (channelDrafts[radio.band] === undefined) channelDrafts[radio.band] = radio.channel;
  }

  onMount(() => {
    let unsubscribe: (() => void) | null = null;
    let disposed = false;
    subscribeSnapshots((next) => snapshot = next).then((stop) => {
      if (disposed) stop(); else unsubscribe = stop;
    }).catch(() => {});
    if (loadSnapshot) loadSnapshot().then((next) => snapshot = next).catch(showFailure);
    const telemetryTimer = window.setInterval(() => {
      if (!operationPending && activeSection === "overview" && snapshot.selectedCandidateId) {
        refreshClients().catch(() => {});
      }
    }, 15000);
    const statusTimer = window.setInterval(() => {
      if (!operationPending) refreshStatus().catch(() => {});
    }, 15000);
    const clockTimer = window.setInterval(() => nowSeconds = Math.floor(Date.now() / 1000), 1000);
    return () => { disposed = true; unsubscribe?.(); window.clearInterval(telemetryTimer); window.clearInterval(statusTimer); window.clearInterval(clockTimer); };
  });

  async function selectCandidate(candidateId: string) {
    try { const next = await onSelectCandidate(candidateId); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function verifyIdentity() {
    try { const next = await onVerify(); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function previewDiagnostics() {
    try { diagnostics = await onPreviewDiagnostics(); exportPath = diagnostics.suggestedName; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function exportDiagnostics() {
    if (!diagnostics) return;
    try { await onExportDiagnostics(exportPath, diagnostics.preview); failure = null; }
    catch (error) { showFailure(error); }
  }

  async function updateFrom(operation: () => Promise<ApplicationSnapshot>) {
    try { snapshot = await operation(); failure = null; }
    catch (error) { showFailure(error); }
  }

  async function refreshClients() {
    refreshingStatus = true;
    try { snapshot = await onRefreshClientEvidence(true); failure = null; }
    catch (error) { showFailure(error); }
    finally { refreshingStatus = false; nowSeconds = Math.floor(Date.now() / 1000); }
  }

  async function refreshStatus() {
    refreshingStatus = true;
    try { snapshot = await onRefreshStatus(); }
    finally { refreshingStatus = false; nowSeconds = Math.floor(Date.now() / 1000); }
  }

  function freshnessText(): string {
    if (refreshingStatus) return "Observed status · Refreshing";
    if (snapshot.observedStatusAt === null) return "Observed status · Not updated yet";
    const age = Math.max(0, nowSeconds - snapshot.observedStatusAt);
    const relative = age < 60 ? `${age}s ago` : `${Math.floor(age / 60)}m ago`;
    return `Observed status · ${age > 45 ? "Delayed · " : ""}Updated ${relative}`;
  }

  async function saveObservedClient(hardwareAddress: string) {
    const label = (clientLabelDrafts[hardwareAddress] ?? "").trim();
    if (!label) return;
    await runMutation(`Saving ${label}`, () => onSaveClientLabel(hardwareAddress, label));
    clientLabelDrafts[hardwareAddress] = "";
  }

  async function renameSavedClient(hardwareAddress: string, currentLabel: string) {
    const label = (savedLabelDrafts[hardwareAddress] ?? currentLabel).trim();
    if (!label) return;
    await runMutation(`Renaming ${currentLabel}`, () => onRenameSavedClient(hardwareAddress, label));
  }

  async function runMutation(label: string, operation: () => Promise<ApplicationSnapshot | void>) {
    operationPending = true;
    pendingLabel = label;
    try { const next = await operation(); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
    finally { operationPending = false; pendingLabel = ""; }
  }

  function showFailure(error: unknown) {
    if (typeof error === "object" && error !== null && "code" in error && "message" in error && "guidance" in error) {
      failure = error as OperationFailure;
    } else {
      failure = { code: "unexpected_failure", message: "The operation could not be completed safely.", guidance: "Refresh current state and review diagnostics before trying again.", diagnosticReference: null };
    }
  }

  async function applyBand(radio: RadioSnapshot) {
    const update: RadioUpdate = {};
    if (ssidDrafts[radio.band] !== radio.ssid) update.ssid = ssidDrafts[radio.band];
    if (channelDrafts[radio.band] !== radio.channel) update.channel = channelDrafts[radio.band];
    if (radio.band === "five_ghz" && fiveGhzPassword) update.password = fiveGhzPassword;
    fiveGhzPassword = "";
    try { await runMutation(`Applying ${bandName(radio.band)} settings`, () => onApplyRadio(radio.band, update)); }
    finally { fiveGhzPassword = ""; }
  }

  const dimensions: Array<{ key: keyof ApplicationSnapshot; label: string }> = [
    { key: "usb", label: "USB link" },
    { key: "sharing", label: "Host sharing" },
    { key: "protocol", label: "Puppis identity" },
    { key: "configuration", label: "Configuration health" },
    { key: "clients", label: "Client evidence" },
  ];

  function statusFor(key: keyof ApplicationSnapshot): StatusDimension {
    return snapshot[key] as StatusDimension;
  }

  function bandName(band: RadioBand): string {
    return band === "five_ghz" ? "5 GHz" : "2.4 GHz";
  }

  function roleName(role: DeviceRole | null): string {
    if (role === "prism_pulse") return "PrismPulse mode";
    if (role === "wifi_hotspot") return "Wi-Fi hotspot mode";
    if (role === "wifi_adapter") return "Wi-Fi adapter mode";
    return "Unknown role";
  }

  function otherSupportedRole(role: DeviceRole): DeviceRole {
    return role === "prism_pulse" ? "wifi_hotspot" : "prism_pulse";
  }

  function openSection(section: Section) {
    activeSection = section;
  }

  function handleHotkey(event: KeyboardEvent) {
    if (!event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const shortcuts: Record<string, Section> = {
      "1": "overview",
      "2": "network",
      "3": "wifi",
      "4": "diagnostics",
      a: "about",
    };
    const section = shortcuts[event.key.toLowerCase()];
    if (section) {
      event.preventDefault();
      openSection(section);
    }
  }

  async function requestManagedSharing() {
    if (snapshot.hostSharing?.profileKind === "external" && snapshot.hostSharing.active) {
      confirmManagedReplacement = true;
      return;
    }
    await runMutation("Enabling managed sharing", onEnableManagedSharing);
  }
</script>

<svelte:window on:keydown={handleHotkey} />

<svelte:head>
  <title>Puppis S1 Manager for Linux</title>
</svelte:head>

<div class="shell" style={`font-size: ${textScale}%`}>
  <header>
    <div>
      <p class="eyebrow">{uiText.unofficial}</p>
      <h1>Puppis S1 Manager</h1>
    </div>
    <div class="header-tools">
      <span class="revision" aria-live="polite">{freshnessText()}</span>
      <label class="text-scale">Text size
        <select aria-label="Text size" bind:value={textScale}>
          <option value={100}>100%</option>
          <option value={125}>125%</option>
          <option value={150}>150%</option>
        </select>
      </label>
      <button class="about-link" type="button" aria-keyshortcuts="Alt+A" aria-current={activeSection === "about" ? "page" : undefined} on:click={() => openSection("about")}>About</button>
    </div>
  </header>

  {#if failure}<div class="failure-banner" role="alert"><div><strong>{failure.message}</strong><p>{failure.guidance}</p></div><button type="button" aria-label="Dismiss operation failure" on:click={() => failure = null}>×</button></div>{/if}
  {#if operationPending}<div class="operation-status" role="status" aria-live="polite">{pendingLabel}. Verification and recovery will finish before other changes are allowed.</div>{/if}

  <nav aria-label="Application sections">
    <button type="button" aria-keyshortcuts="Alt+1" aria-current={activeSection === "overview" ? "page" : undefined} on:click={() => openSection("overview")}>Overview</button>
    <button type="button" aria-keyshortcuts="Alt+2" aria-current={activeSection === "network" ? "page" : undefined} on:click={() => openSection("network")}>Network</button>
    <button type="button" aria-keyshortcuts="Alt+3" aria-current={activeSection === "wifi" ? "page" : undefined} on:click={() => openSection("wifi")}>Wi-Fi</button>
    <button type="button" aria-keyshortcuts="Alt+4" aria-current={activeSection === "diagnostics" ? "page" : undefined} on:click={() => openSection("diagnostics")}>Diagnostics</button>
  </nav>
  <p class="sr-only" aria-live="polite">Current section: {activeSection === "wifi" ? "Wi-Fi" : activeSection[0].toUpperCase() + activeSection.slice(1)}</p>

  {#if activeSection === "overview"}
  <main id="overview">
    <div class="section-heading">
      <div>
        <p class="eyebrow">Current connection</p>
        <h2>Overview</h2>
      </div>
      <p>The manager observes first. Nothing changes until you request it.</p>
    </div>

    {#if !snapshot.verifiedPuppis}
      <section class="first-launch" aria-labelledby="first-launch-heading">
        <div><p class="eyebrow">Read first</p><h3 id="first-launch-heading">{uiText.firstLaunchHeading}</h3><p>{uiText.firstLaunchIntro}</p></div>
        <ol>
          <li class:complete={snapshot.candidates.length > 0}><strong>Connect and select a Puppis candidate.</strong><span>{snapshot.candidates.length === 0 ? "Connect the original P1411 over USB." : snapshot.selectedCandidateId ? "Candidate selected; protocol identity is still unverified." : "Choose the adapter connected to your Puppis."}</span></li>
          <li class:complete={snapshot.verifiedPuppis !== null}><strong>Verify P1411 identity.</strong><span>Verification is read-only and must succeed before device changes are offered.</span></li>
          <li><strong>Review host sharing.</strong><span>Network authorization is requested only after the Network page explains the managed sharing profile.</span><button class="secondary" type="button" on:click={() => openSection("network")}>Review network explanation</button></li>
          <li><strong>Keep diagnostics available.</strong><span>A private preview remains available without enabling mutations.</span><button class="secondary" type="button" on:click={() => openSection("diagnostics")}>Open Diagnostics</button></li>
        </ol>
      </section>
    {/if}

    <section class="status-grid" aria-label="Operational status">
      {#each dimensions as dimension}
        {@const status = statusFor(dimension.key)}
        <article class:attention={status.level === "attention"}>
          <div class="status-title">
            <span class={`dot ${status.level}`} aria-hidden="true"></span>
            <h3>{dimension.label}</h3>
            <span class="status-level">{status.level}</span>
          </div>
          <p>{status.summary}</p>
          {#if status.guidance}<small>{status.guidance}</small>{/if}
        </article>
      {/each}
    </section>

    <section class="client-panel" aria-labelledby="client-evidence-heading">
        <div class="client-panel-heading"><div><p class="eyebrow">Passive local evidence</p><h3 id="client-evidence-heading">Observed clients</h3></div><button class="secondary" type="button" disabled={refreshingStatus} on:click={refreshClients}>Refresh observed clients</button></div>
        <p>A client appears after it sends local network traffic. This passive view does not probe, ping, wake, or prove Wi-Fi association.</p>
        {#if snapshot.clientEvidence.length > 0}
        <div class="client-list">
          {#each snapshot.clientEvidence as client}
            {@const savedRecord = snapshot.savedClients.find((saved) => saved.hardwareAddress === client.hardwareAddress)}
            <div class="client-row">
              <strong>{client.displayName}</strong>
              <span>{client.kind === "connected" ? "Connected client" : "Recently observed client"}</span>
              <small>{client.hardwareAddress}</small>
              <small>{client.addresses.length > 0 ? client.addresses.join(" · ") : "No current IP address observed"}</small>
              {#if client.kind === "recently_observed"}<small>Host network evidence; current Wi-Fi connection is not confirmed.</small>{:else}<small>Currently reported active by a validated Puppis capability.</small>{/if}
              {#if !client.saved && snapshot.savedClientsAvailable}
                <form class="client-label-form" on:submit|preventDefault={() => saveObservedClient(client.hardwareAddress)}>
                  <label>Label <input aria-label={`Label ${client.hardwareAddress}`} maxlength="40" bind:value={clientLabelDrafts[client.hardwareAddress]} /></label>
                  <button class="secondary" type="submit" disabled={!clientLabelDrafts[client.hardwareAddress]?.trim()} aria-label={`Save ${clientLabelDrafts[client.hardwareAddress]?.trim() || "client label"}`}>Save label</button>
                </form>
                {#if snapshot.savedClients.length > 0}
                  <div class="button-row">
                    {#each snapshot.savedClients as saved}
                      <button class="secondary" type="button" on:click={() => reassociation = { previousHardwareAddress: saved.hardwareAddress, newHardwareAddress: client.hardwareAddress }}>Use saved label “{saved.label}”</button>
                    {/each}
                  </div>
                {/if}
              {:else if savedRecord && snapshot.savedClientsAvailable}
                <small>Last observed {new Date(savedRecord.lastObservedAt * 1000).toLocaleString()}</small>
                <form class="client-label-form" on:submit|preventDefault={() => renameSavedClient(savedRecord.hardwareAddress, savedRecord.label)}>
                  <label>Rename <input aria-label={`Rename ${savedRecord.label}`} maxlength="40" value={savedLabelDrafts[savedRecord.hardwareAddress] ?? savedRecord.label} on:input={(event) => savedLabelDrafts[savedRecord.hardwareAddress] = event.currentTarget.value} /></label>
                  <button class="secondary" type="submit">Rename</button>
                  <button class="danger" type="button" aria-label={`Forget ${savedRecord.label}`} on:click={() => runMutation(`Forgetting ${savedRecord.label}`, () => onForgetSavedClient(savedRecord.hardwareAddress))}>Forget</button>
                </form>
              {/if}
            </div>
          {/each}
        </div>
        {:else}<p>No clients are currently observed.</p>{/if}
      </section>

      {#if unobservedSavedClients.length > 0 || !snapshot.savedClientsAvailable}
        <section class="client-panel" aria-labelledby="saved-clients-heading">
          <div class="client-panel-heading"><div><p class="eyebrow">Recognition only</p><h3 id="saved-clients-heading">Saved clients</h3></div>{#if snapshot.savedClients.length > 0}<button class="danger" type="button" on:click={() => confirmClearSaved = true}>Clear all saved clients</button>{/if}</div>
          {#if !snapshot.savedClientsAvailable}
            <div class="recovery" role="alert"><p>{snapshot.savedClientsFailure?.message}</p><p>{snapshot.savedClientsFailure?.guidance}</p><button class="danger" type="button" on:click={() => confirmResetSaved = true}>Reset saved clients</button></div>
          {/if}
          <div class="client-list">
            {#each unobservedSavedClients as saved}
              <div class="client-row"><strong>{saved.label}</strong><span>Saved · not currently observed</span><small>{saved.hardwareAddress}</small><small>{saved.addresses.length > 0 ? saved.addresses.join(" · ") : "No last observed IP address"}</small><small>Last observed {new Date(saved.lastObservedAt * 1000).toLocaleString()}</small>
                <form class="client-label-form" on:submit|preventDefault={() => renameSavedClient(saved.hardwareAddress, saved.label)}><label>Rename <input aria-label={`Rename ${saved.label}`} maxlength="40" value={savedLabelDrafts[saved.hardwareAddress] ?? saved.label} on:input={(event) => savedLabelDrafts[saved.hardwareAddress] = event.currentTarget.value} /></label><button class="secondary" type="submit">Rename</button><button class="danger" type="button" on:click={() => runMutation(`Forgetting ${saved.label}`, () => onForgetSavedClient(saved.hardwareAddress))}>Forget</button></form>
              </div>
            {/each}
          </div>
        </section>
      {/if}

    {#if snapshot.candidates.length > 0 && !snapshot.selectedCandidateId}
      <section class="candidate-panel" aria-labelledby="candidate-heading">
        <div>
          <p class="eyebrow">Selection required</p>
          <h3 id="candidate-heading">Which adapter is connected to your Puppis?</h3>
          <p>Candidate identity has not been verified.</p>
        </div>
        <div class="candidate-list">
          {#each snapshot.candidates as candidate}
            <button type="button" on:click={() => selectCandidate(candidate.id)} aria-label={`Inspect ${candidate.displayName}`}>
              <strong>{candidate.displayName}</strong>
              <span>{candidate.linkSpeed === "super_speed" ? "SuperSpeed" : candidate.linkSpeed === "usb2" ? "USB 2" : "Speed unknown"}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    {#if snapshot.selectedCandidateId && !snapshot.verifiedPuppis}
      <section class="candidate-panel" aria-label="P1411 identity verification">
        <div>
          <p class="eyebrow">Read-only verification</p>
          <h3>Verify this candidate</h3>
          <p>USB identity alone does not prove this is a P1411.</p>
        </div>
        <div class="action-column">
          <button class="primary" type="button" on:click={verifyIdentity}>Verify P1411 identity</button>
          <small>No device setting will be changed.</small>
        </div>
      </section>
    {/if}

    {#if snapshot.verifiedPuppis}
      <section class="device-facts" aria-label="Verified Puppis details">
        <span><small>Model</small><strong>{snapshot.verifiedPuppis.model}</strong></span>
        <span><small>Firmware</small><strong>{snapshot.verifiedPuppis.firmware}</strong></span>
        <span><small>Device changes</small><strong>{snapshot.mutationsQualified ? "Qualified" : "Read-only"}</strong></span>
      </section>
    {/if}
  </main>
  {:else if activeSection === "network"}
    <main>
      <div class="section-heading"><div><p class="eyebrow">Host networking</p><h2>Network</h2></div><p>NetworkManager remains in control of routing, DHCP, DNS, NAT, and firewall behavior.</p></div>
      <button class="secondary" type="button" on:click={() => updateFrom(onInspectNetwork)}>Inspect NetworkManager state</button>
      {#if snapshot.hostSharing}
        <section class="network-card" aria-label="Host sharing details">
          <div><small>Profile ownership</small><strong>{snapshot.hostSharing.profileKind === "external" ? "External sharing profile" : snapshot.hostSharing.profileKind === "managed" ? "Managed sharing profile" : "No sharing profile"}</strong></div>
          <div><small>Profile</small><strong>{snapshot.hostSharing.profileName ?? "Not configured"}</strong></div>
          <div><small>Upstream</small><strong>{snapshot.hostSharing.upstreamAvailable ? snapshot.hostSharing.upstreamDescription ?? "Available" : "Unavailable"}</strong></div>
          <div><small>Downstream</small><strong>{snapshot.hostSharing.downstreamAddress ?? "Not configured"}</strong></div>
        </section>
        {#if snapshot.hostSharing.subnetConflict}<p class="warning">Resolve the Puppis subnet conflict before enabling managed sharing.</p>{/if}
        {#if snapshot.hostSharing.profileKind !== "managed"}
          <div class="network-action"><p>{snapshot.hostSharing.profileKind === "external" && snapshot.hostSharing.active ? "Enabling managed sharing will replace the active external downstream configuration. The external profile will not be modified or deleted." : "Enabling creates a system-wide app-owned profile after verification."} A normal desktop authorization prompt may appear.</p><button class="primary" type="button" disabled={operationPending || snapshot.hostSharing.subnetConflict || !snapshot.hostSharing.upstreamAvailable} on:click={requestManagedSharing}>Enable managed sharing</button></div>
        {:else}
          <div class="button-row">
            <button class="secondary" type="button" disabled={operationPending} on:click={() => runMutation("Disabling managed sharing", onDisableManagedSharing)}>Disable managed sharing</button>
            <button class="danger" type="button" disabled={operationPending} on:click={() => runMutation("Removing managed sharing", onRemoveManagedSharing)}>Remove managed sharing</button>
          </div>
        {/if}
      {/if}
      {#if confirmManagedReplacement}
        <div class="dialog-backdrop">
          <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="sharing-confirm-title" tabindex="-1">
            <p class="eyebrow">Replace active downstream configuration</p><h3 id="sharing-confirm-title">Enable managed sharing instead?</h3>
            <p>The external profile remains unchanged, but the app-owned profile will become active on this Puppis interface.</p>
            <div class="button-row"><button class="secondary" type="button" on:click={() => confirmManagedReplacement = false}>Keep external sharing</button><button class="primary" type="button" on:click={async () => { confirmManagedReplacement = false; await runMutation("Enabling managed sharing", onEnableManagedSharing); }}>Replace with managed sharing</button></div>
          </div>
        </div>
      {/if}
    </main>
  {:else if activeSection === "wifi"}
    <main>
      <div class="section-heading"><div><p class="eyebrow">Device configuration</p><h2>Wi-Fi</h2></div><p>Settings are read on demand. Existing passwords never enter this view.</p></div>
      {#if snapshot.recoveryRequired}<div class="recovery" role="alert"><p>Recovery required. Device mutations are locked until the current state is reconciled.</p><button class="secondary" type="button" on:click={() => updateFrom(onAcceptRecoveryBaseline)}>Read and accept current configuration as baseline</button></div>{/if}
      <button class="secondary" type="button" on:click={() => updateFrom(onReadRadios)}>Read current radio settings</button>
      {#if snapshot.deviceRole}
        <section class="role-card" aria-label="Device role">
          <div><small>Current role</small><strong>{roleName(snapshot.deviceRole)}</strong>{#if snapshot.deviceRole === "wifi_adapter"}<p>Wi-Fi adapter mode is recognized for diagnostics but cannot be selected.</p>{/if}</div>
          {#if snapshot.deviceRole !== "wifi_adapter"}
            <button class="secondary" type="button" disabled={operationPending || snapshot.recoveryRequired || !snapshot.mutationsQualified} on:click={() => transitionTarget = otherSupportedRole(snapshot.deviceRole!)}>Switch to {roleName(otherSupportedRole(snapshot.deviceRole))}</button>
          {/if}
        </section>
      {/if}
      <div class="radio-grid">
        {#each snapshot.radios as radio}
          <section class="radio-card" aria-label={`${bandName(radio.band)} settings`}>
            <div class="radio-heading"><div><p class="eyebrow">{bandName(radio.band)}</p><h3>{radio.ssid}</h3></div><span>{radio.country} · {radio.bandwidth} MHz</span></div>
            <form on:submit|preventDefault={() => applyBand(radio)}>
              <label>{bandName(radio.band)} network name<input aria-label={`${bandName(radio.band)} network name`} bind:value={ssidDrafts[radio.band]} disabled={operationPending || !radio.ssidMutation || snapshot.recoveryRequired} /></label>
              <label>{bandName(radio.band)} channel<select aria-label={`${bandName(radio.band)} channel`} bind:value={channelDrafts[radio.band]} disabled={operationPending || radio.qualifiedChannels.length === 0 || snapshot.recoveryRequired}>{#each radio.qualifiedChannels as channel}<option value={channel}>{channel === "0" ? "Automatic" : channel}</option>{/each}</select></label>
              {#if radio.passwordMutation && radio.band === "five_ghz"}
                <label>New 5 GHz password<input aria-label="New 5 GHz password" type="password" autocomplete="new-password" bind:value={fiveGhzPassword} disabled={operationPending || snapshot.recoveryRequired} /></label>
              {:else if radio.passwordUnavailableReason}
                <p class="capability-note">{radio.passwordUnavailableReason}</p>
              {/if}
              <button class="primary" type="submit" disabled={operationPending || snapshot.recoveryRequired || (!radio.ssidMutation && radio.qualifiedChannels.length === 0)}>Apply {bandName(radio.band)} settings</button>
            </form>
          </section>
        {/each}
      </div>
      {#if snapshot.radios.length === 0}<p>Verify a Puppis, then read current radio settings.</p>{/if}
      {#if transitionTarget}
        <div class="dialog-backdrop">
          <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="role-confirm-title" tabindex="-1">
            <p class="eyebrow">Disruptive device change</p><h3 id="role-confirm-title">Switch from {roleName(snapshot.deviceRole)} to {roleName(transitionTarget)}?</h3>
            <p>Connected clients will disconnect during this transition. The new role is reported only after read-back verification.</p>
            <div class="button-row"><button class="secondary" type="button" on:click={() => transitionTarget = null}>Cancel</button><button class="primary" type="button" disabled={operationPending} on:click={async () => { const target = transitionTarget!; transitionTarget = null; await runMutation("Switching device role", () => onSetDeviceRole(target, true)); }}>Confirm role transition</button></div>
          </div>
        </div>
      {/if}
    </main>
  {:else if activeSection === "diagnostics"}
    <main>
      <div class="section-heading">
        <div><p class="eyebrow">Private by construction</p><h2>Diagnostics</h2></div>
        <p>Review the complete redacted artifact before choosing where to save it.</p>
      </div>
      <button class="primary" type="button" on:click={previewDiagnostics}>Generate private preview</button>
      {#if diagnostics}
        <pre class="diagnostics-preview" aria-label="Diagnostics preview">{diagnostics.preview}</pre>
        <label class="export-field">Export destination<input bind:value={exportPath} /></label>
        <button class="primary" type="button" on:click={exportDiagnostics}>Export reviewed diagnostics</button>
      {/if}
    </main>
  {:else}
    <main>
      <div class="section-heading"><div><p class="eyebrow">Project information</p><h2>About</h2></div><p>Version {packageMetadata.version}</p></div>
      <section class="about-card" aria-label="Application attribution and licensing">
        <h3>Puppis S1 Manager for Linux</h3>
        <p>This is an unofficial application for the original PrismXR Puppis S1 (P1411). PrismXR does not endorse or support this project.</p>
        <p>{uiText.attribution}</p>
        <p>The original project is copyright © 2026 Keary Chang and is licensed under the MIT License. Vendor binaries, artwork, raw traces, and decompiled source are excluded from the project and package.</p>
      </section>
    </main>
  {/if}
  {#if confirmClearSaved}
    <div class="dialog-backdrop"><div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="clear-saved-title"><h3 id="clear-saved-title">Clear all saved clients?</h3><p>Every saved label, hardware address, last-observed time, and latest address set will be removed.</p><div class="button-row"><button class="secondary" type="button" on:click={() => confirmClearSaved = false}>Cancel</button><button class="danger" type="button" on:click={async () => { confirmClearSaved = false; await runMutation("Clearing saved clients", () => onClearSavedClients(true)); }}>Clear saved clients</button></div></div></div>
  {/if}
  {#if confirmResetSaved}
    <div class="dialog-backdrop"><div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="reset-saved-title"><h3 id="reset-saved-title">Reset unreadable saved-client data?</h3><p>The existing file will be replaced with an empty, versioned saved-client file.</p><div class="button-row"><button class="secondary" type="button" on:click={() => confirmResetSaved = false}>Cancel</button><button class="danger" type="button" on:click={async () => { confirmResetSaved = false; await runMutation("Resetting saved clients", () => onResetSavedClients(true)); }}>Reset saved clients</button></div></div></div>
  {/if}
  {#if reassociation}
    {@const previous = snapshot.savedClients.find((saved) => saved.hardwareAddress === reassociation?.previousHardwareAddress)}
    <div class="dialog-backdrop"><div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="reassociate-title"><h3 id="reassociate-title">Move saved label to this hardware address?</h3><p>Label: {previous?.label}</p><p>Old MAC: {reassociation.previousHardwareAddress}<br />New MAC: {reassociation.newHardwareAddress}<br />Last observed: {previous ? new Date(previous.lastObservedAt * 1000).toLocaleString() : "Unknown"}</p><div class="button-row"><button class="secondary" type="button" on:click={() => reassociation = null}>Cancel</button><button class="primary" type="button" on:click={async () => { const request = reassociation!; reassociation = null; await runMutation("Reassociating saved client", () => onReassociateSavedClient(request.previousHardwareAddress, request.newHardwareAddress, true)); }}>Confirm reassociation</button></div></div></div>
  {/if}
  <footer><p>{uiText.shortcuts}</p></footer>
</div>
