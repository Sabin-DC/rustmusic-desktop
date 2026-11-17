<script lang="ts">
  // Mesure de diagnostic : images par seconde réellement obtenues par la page.
  // Ctrl+Alt+F l'affiche ou la masque. Éteint par défaut, aucune boucle tant qu'il est masqué.
  let visible = $state(false);
  let fps = $state(0);
  let pire = $state(0); // plus longue image de la dernière seconde, en ms

  // Ventilation de la pire image lente de la dernière seconde (Chromium : Long Animation Frames).
  // Dit où part le temps : scripts, calcul de style et de disposition, ou dessin.
  let lente = $state("");

  $effect(() => {
    if (!visible || typeof PerformanceObserver === "undefined") return;
    let pireImage: any = null;
    let obs: PerformanceObserver | null = null;
    try {
      obs = new PerformanceObserver((liste) => {
        for (const e of liste.getEntries() as any[]) {
          if (!pireImage || e.duration > pireImage.duration) pireImage = e;
        }
      });
      obs.observe({ type: "long-animation-frame", buffered: false });
    } catch {
      lente = "LoAF indisponible";
      return;
    }
    const id = setInterval(() => {
      const e = pireImage;
      pireImage = null;
      if (!e) { lente = ""; return; }
      const scripts = (e.scripts ?? []).reduce((s: number, x: any) => s + x.duration, 0);
      const styleEtDispo = e.startTime + e.duration - (e.styleAndLayoutStart ?? e.startTime + e.duration);
      const rendu = Math.max(0, (e.styleAndLayoutStart ?? e.renderStart) - e.renderStart);
      const source = (e.scripts ?? []).sort((a: any, b: any) => b.duration - a.duration)[0];
      const nom = source ? (source.invoker || source.sourceURL || "").toString().slice(-40) : "";
      lente = `lente ${Math.round(e.duration)} ms · scripts ${Math.round(scripts)} · rendu ${Math.round(rendu)} · style+dispo ${Math.round(styleEtDispo)}${nom ? " · " + nom : ""}`;
    }, 1000);
    return () => { clearInterval(id); obs?.disconnect(); lente = ""; };
  });

  $effect(() => {
    if (!visible) return;
    let raf = 0;
    let last = performance.now();
    let debut = last;
    let n = 0;
    let max = 0;
    const loop = (now: number) => {
      max = Math.max(max, now - last);
      last = now;
      n++;
      if (now - debut >= 1000) {
        fps = Math.round((n * 1000) / (now - debut));
        pire = Math.round(max);
        n = 0;
        max = 0;
        debut = now;
      }
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      visible = !visible;
    }
  }
</script>

<svelte:window {onkeydown} />

{#if visible}
  <div class="fixed bottom-2 left-2 z-9999 px-2 py-1 rounded bg-black/80 text-white font-mono text-xs pointer-events-none">
    {fps} fps · pire {pire} ms
    {#if lente}<div class="mt-0.5 text-amber-300">{lente}</div>{/if}
  </div>
{/if}
