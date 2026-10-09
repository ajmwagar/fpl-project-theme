/* Small browser boundary: copy the canonical Markdown export, never scraped HTML. */
for (const button of document.querySelectorAll('[data-copy-markdown]')) {
  button.addEventListener('click', async () => {
    const status = button.parentElement.querySelector('[data-copy-status]');
    status.setAttribute('role', 'status');
    button.disabled = true;
    status.textContent = 'Fetching Markdown…';
    try {
      const response = await fetch(button.dataset.copyMarkdown, { credentials: 'same-origin' });
      if (!response.ok) throw new Error(`Markdown request failed (${response.status})`);
      const markdown = await response.text();
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable in this browser or insecure connection');
      await navigator.clipboard.writeText(markdown);
      status.textContent = 'Markdown copied.';
    } catch (error) {
      status.textContent = `Could not copy: ${error.message}. Use Download Markdown, then select and copy its contents.`;
      status.setAttribute('role', 'alert');
    } finally {
      button.disabled = false;
    }
  });
}
