// Appearance › Text: Mac (Inter, the default) or Windows (Segoe UI). Read
// before the page draws so text never switches font; app.js stores the choice.
try {
  if (localStorage.getItem('blazon.type') === 'windows') document.documentElement.classList.add('win-type');
} catch {}
