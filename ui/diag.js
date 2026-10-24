// Sends errors from this page to the Blazon log, for problem reports.
(() => {
  const page = location.pathname.split('/').pop() || 'index.html';
  const send = (message, stack) => {
    try { window.__TAURI__.core.invoke('report_js_error', { page, message: String(message).slice(0, 500), stack: stack ? String(stack) : null }); } catch {}
  };
  addEventListener('error', (e) => send(e.message, e.error && e.error.stack));
  addEventListener('unhandledrejection', (e) => send(`promessa rejeitada: ${e.reason && (e.reason.message || e.reason)}`, e.reason && e.reason.stack));
})();
