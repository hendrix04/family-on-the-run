<script lang="ts">
  let { name, size = 22 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    home: 'm3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1Z',
    calendar: 'M8 2v4m8-4v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2ZM7 14h2m3 0h2m3 0h1M7 18h2m3 0h2',
    month: 'M8 2v4m8-4v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2ZM8 15h8m-8 3h4',
    food: 'M5 3v7m3-7v7m-6-7v7h6M5 10v11M18 3c-3 2-4 6-4 10h4m0-10v18',
    house: 'm3 10 9-7 9 7M5 9v12h14V9M9 21v-7h6v7',
    plus: 'M12 5v14M5 12h14',
    chevronLeft: 'm15 6-6 6 6 6',
    chevronRight: 'm9 6 6 6-6 6',
    check: 'm5 12 4 4L19 6',
    edit: 'm15 5 4 4M4 16l-1 5 5-1L21 7l-4-4Z',
    trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7',
    close: 'm6 6 12 12M6 18 18 6',
    clock: 'M12 8v4l3 2M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Z',
    location: 'M12 21s7-6 7-12A7 7 0 0 0 5 9c0 6 7 12 7 12ZM12 6a3 3 0 1 0 0 6 3 3 0 0 0 0-6Z',
    logout: 'M9 3H4v18h5m6-14 5 5-5 5M8 12h12',
    settings: 'M4 6h16M4 12h16M4 18h16M8 3v6m8 0v6M10 15v6',
    lock: 'M7 10V7a5 5 0 0 1 10 0v3M5 10h14v11H5ZM12 14v3',
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={paths[name] ?? paths.home} />
</svg>
