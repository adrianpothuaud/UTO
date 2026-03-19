document.addEventListener("DOMContentLoaded", () => {
  const revealNodes = document.querySelectorAll("[data-reveal]");

  if (!revealNodes.length) {
    return;
  }

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    revealNodes.forEach((node) => node.classList.add("reveal-visible"));
    return;
  }

  const observer = new IntersectionObserver(
    (entries, observerInstance) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) {
          return;
        }

        entry.target.classList.add("reveal-visible");
        observerInstance.unobserve(entry.target);
      });
    },
    {
      threshold: 0.15,
      rootMargin: "0px 0px -40px 0px",
    }
  );

  revealNodes.forEach((node, index) => {
    node.style.transitionDelay = `${Math.min(index * 80, 400)}ms`;
    observer.observe(node);
  });
});
