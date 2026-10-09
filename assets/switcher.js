/* Shared project menu: hover preview, native click toggle, outside dismissal. */
for (const menu of document.querySelectorAll('.project-switcher')) {
  if (menu.dataset.initialized) continue;
  menu.dataset.initialized = 'true';
  menu.addEventListener('pointerenter', event => {
    if (event.pointerType === 'mouse') menu.open = true;
  });
  menu.addEventListener('pointerleave', event => {
    if (event.pointerType === 'mouse') menu.open = false;
  });
  document.addEventListener('pointerdown', event => {
    if (!menu.contains(event.target)) menu.open = false;
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && menu.open) {
      menu.open = false;
      menu.querySelector('summary').focus();
    }
  });
  menu.addEventListener('focusout', event => {
    if (!menu.contains(event.relatedTarget)) menu.open = false;
  });
}
