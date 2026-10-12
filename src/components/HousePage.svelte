<script lang="ts">
  import Icon from './Icon.svelte';
  const sections = [
    { label: 'Every day', tasks: ['Clear the kitchen counters', 'Wash dishes and wipe the sink', 'A quick tidy of shared spaces'] },
    { label: 'Every week', tasks: ['Vacuum and mop floors', 'Clean the bathrooms', 'Wash bedding and towels', 'Take out trash and recycling'] },
    { label: 'Every month', tasks: ['Check the HVAC filter', 'Wipe appliances and cabinet fronts', 'Clean out the refrigerator'] },
    { label: 'Every season', tasks: ['Check smoke and carbon monoxide alarms', 'Clean windows and screens', 'Review outdoor maintenance'] },
  ];
</script>

<div class="page-intro"><p class="eyebrow">A PLACE FOR THE ROUTINE</p><h2>A little care goes a long way.</h2><p>A starter schedule for the house. Adjust these routines as you make this dashboard your own.</p></div>
<div class="house-grid">
  {#each sections as section}
    <section class="house-card"><span class="house-icon"><Icon name="house" /></span><h3>{section.label}</h3><ul>{#each section.tasks as task}<li><span class="routine-dot"></span>{task}</li>{/each}</ul></section>
  {/each}
</div>
