<script lang="ts">
  import { onMount, tick } from 'svelte';

  type TextField = HTMLInputElement | HTMLTextAreaElement;
  let keyboard: HTMLDivElement;
  let target = $state<TextField | null>(null);
  let shifted = $state(false);
  let symbols = $state(false);
  const rows = $derived(symbols
    ? ['1234567890', '@#$%&*()[]', ':;/\\!?+=_', '\'".,-<>{}']
    : ['1234567890', 'qwertyuiop', 'asdfghjkl', 'zxcvbnm']);

  function textField(element: EventTarget | null): element is TextField {
    return (element instanceof HTMLTextAreaElement ||
      element instanceof HTMLInputElement && ['text', 'password', 'email', 'url', 'search', 'tel'].includes(element.type))
      && !element.disabled && !element.readOnly;
  }

  function hide() {
    target?.blur();
    keyboard.hidePopover();
    target = null;
    shifted = false;
    symbols = false;
    document.documentElement.style.removeProperty('--keyboard-height');
  }

  async function focus(event: FocusEvent) {
    if (!matchMedia('(min-width: 900px)').matches) return;
    if (!textField(event.target)) return;
    target = event.target;
    // Keep the keyboard inside a modal's focus scope, and above it in the top layer.
    const parent = target.closest('dialog') ?? document.body;
    if (keyboard.parentElement !== parent) parent.append(keyboard);
    await tick();
    if (!target?.isConnected) return;
    keyboard.showPopover();
    document.documentElement.style.setProperty('--keyboard-height', `${keyboard.offsetHeight + 16}px`);
    target.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }

  function key(value: string) {
    if (!target || !textField(target)) { hide(); return; }
    const field = target;
    if (value === 'Done') { field.blur(); hide(); return; }
    if (value === 'Shift') { shifted = !shifted; return; }
    if (value === 'Symbols') { symbols = !symbols; return; }
    if (value === 'Enter' && field instanceof HTMLInputElement) {
      field.form?.requestSubmit();
      return;
    }
    const start = field.selectionStart ?? field.value.length;
    const end = field.selectionEnd ?? start;
    const deletion = value === 'Backspace';
    const from = deletion && start === end
      ? start - (Array.from(field.value.slice(0, start)).at(-1)?.length ?? 0) : start;
    const text = deletion ? '' : value === 'Space' ? ' ' : value === 'Enter' ? '\n' : shifted ? value.toUpperCase() : value;
    if (field.maxLength >= 0 && field.value.length - (end - from) + text.length > field.maxLength) return;
    // Email inputs have no selection API; the others preserve the cursor and selection.
    if (field instanceof HTMLInputElement && field.type === 'email') field.value = field.value.slice(0, from) + text + field.value.slice(end);
    else field.setRangeText(text, from, end, 'end');
    field.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: deletion ? 'deleteContentBackward' : 'insertText', data: text }));
    if (!deletion && value !== 'Space') shifted = false;
  }

  onMount(() => {
    let pressedTarget: TextField | null = null;
    const press = () => { pressedTarget = target; };
    const outside = (event: MouseEvent) => {
      if (target && target === pressedTarget && event.target instanceof Node && !keyboard.contains(event.target) && !textField(event.target)) hide();
    };
    const escape = (event: KeyboardEvent) => {
      if (target && event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); hide(); }
    };
    const resize = () => { if (!matchMedia('(min-width: 900px)').matches) hide(); };
    const observer = new MutationObserver(() => { if (target && (!target.isConnected || target.disabled)) hide(); });
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['disabled'] });
    document.addEventListener('focusin', focus);
    document.addEventListener('pointerdown', press, true);
    document.addEventListener('click', outside);
    document.addEventListener('keydown', escape, true);
    window.addEventListener('resize', resize);
    return () => {
      observer.disconnect();
      document.removeEventListener('focusin', focus);
      document.removeEventListener('pointerdown', press, true);
      document.removeEventListener('click', outside);
      document.removeEventListener('keydown', escape, true);
      window.removeEventListener('resize', resize);
      hide();
      keyboard.remove();
    };
  });
</script>

<div bind:this={keyboard} popover="manual" class="touch-keyboard" role="group" aria-label="On-screen keyboard">
  <header><strong>On-screen keyboard</strong><button type="button" onclick={() => key('Done')}>Done</button></header>
  {#each rows as row}
    <div class="key-row">
      {#each Array.from(row) as letter}
        <button type="button" onpointerdown={event => event.preventDefault()} onclick={() => key(letter)}>{shifted ? letter.toUpperCase() : letter}</button>
      {/each}
    </div>
  {/each}
  <div class="key-row">
    <button type="button" aria-pressed={shifted} onpointerdown={event => event.preventDefault()} onclick={() => key('Shift')}>Shift</button>
    <button type="button" aria-pressed={symbols} onpointerdown={event => event.preventDefault()} onclick={() => key('Symbols')}>{symbols ? 'ABC' : 'Symbols'}</button>
    <button type="button" class="space-key" onpointerdown={event => event.preventDefault()} onclick={() => key('Space')}>Space</button>
    <button type="button" onpointerdown={event => event.preventDefault()} onclick={() => key('Backspace')}>Backspace</button>
    <button type="button" onpointerdown={event => event.preventDefault()} onclick={() => key('Enter')}>Enter</button>
  </div>
</div>

<style>
  .touch-keyboard { position: fixed; inset: auto 0 12px; margin: 0 auto; width: min(1050px, calc(100vw - 32px)); padding: 16px; border: 1px solid var(--line); border-radius: 18px; background: #edf1e9; color: var(--green); box-shadow: 0 6px 40px #284c4333; }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .key-row { display: flex; gap: 8px; margin-top: 8px; }
  button { flex: 1; min-width: 0; min-height: 54px; border-radius: 9px; background: #fffefa; color: var(--green); font-size: 20px; box-shadow: 0 2px 0 #d0d8cc; }
  button:active, button[aria-pressed='true'] { background: #c9dbcd; }
  header button { flex: none; min-height: 44px; padding: 0 20px; font-size: 16px; }
  .space-key { flex: 3; }
</style>
