// JS glue — will wire DOM events for WASM
export function setupEventListeners(elementId, eventType, callback) {
  const el = document.getElementById(elementId);
  if (el) el.addEventListener(eventType, callback);
}
