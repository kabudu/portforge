document.addEventListener("DOMContentLoaded", () => {
  const header = document.querySelector("[data-header]");
  const nav = document.querySelector("[data-nav]");
  const navToggle = document.querySelector("[data-nav-toggle]");
  const commandTabs = Array.from(
    document.querySelectorAll("[data-command-tab]"),
  );
  const commandOutput = document.querySelector("[data-command-output]");
  const installTabs = Array.from(
    document.querySelectorAll("[data-install-tab]"),
  );
  const installPanels = Array.from(
    document.querySelectorAll("[data-install-panel]"),
  );
  const copyButtons = Array.from(
    document.querySelectorAll("[data-copy-target]"),
  );
  const copyStatus = document.querySelector("[data-copy-status]");

  const commandExamples = {
    default: [
      "$ portforge",
      "",
      "PORT   PID     PROCESS   PROJECT        GIT           HEALTH",
      "3000   18452   node      next-router    feat/auth     Healthy",
      "5000   20117   python    api_v2         main          Healthy",
      "8080   12931   node      server.js      dirty         Unknown",
    ].join("\n"),
    all: [
      "$ portforge --all",
      "",
      "PORT   PID     PROCESS     CONTEXT",
      "22     800     sshd        system",
      "5432   1540    postgres    data store",
      "3000   18452   node        next-router",
      "5000   20117   python      api_v2",
    ].join("\n"),
    inspect: [
      "$ portforge inspect 3000",
      "",
      "Process: node (PID 18452)",
      "Project: next-router",
      "Framework: Next.js",
      "Branch: feat/auth",
      "Health: 200 OK in 14ms",
      "Network: 127.0.0.1:3000",
    ].join("\n"),
  };

  const setHeaderState = () => {
    if (!header) {
      return;
    }

    header.classList.toggle("is-scrolled", window.scrollY > 12);
  };

  const closeNav = () => {
    if (!nav || !navToggle) {
      return;
    }

    nav.classList.remove("is-open");
    navToggle.setAttribute("aria-expanded", "false");
  };

  const openNav = () => {
    if (!nav || !navToggle) {
      return;
    }

    nav.classList.add("is-open");
    navToggle.setAttribute("aria-expanded", "true");
  };

  if (nav && navToggle) {
    navToggle.addEventListener("click", () => {
      const expanded = navToggle.getAttribute("aria-expanded") === "true";
      if (expanded) {
        closeNav();
      } else {
        openNav();
      }
    });

    nav.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", () => {
        if (window.innerWidth <= 860) {
          closeNav();
        }
      });
    });
  }

  const setCommand = (commandKey, focusPanel = false) => {
    if (!commandOutput || !(commandKey in commandExamples)) {
      return;
    }

    commandTabs.forEach((tab) => {
      const isActive = tab.dataset.commandKey === commandKey;
      tab.classList.toggle("is-active", isActive);
      tab.setAttribute("aria-selected", String(isActive));
    });

    const activeTab = commandTabs.find(
      (tab) => tab.dataset.commandKey === commandKey,
    );
    if (activeTab) {
      commandOutput.setAttribute("aria-labelledby", activeTab.id);
    }

    commandOutput.textContent = commandExamples[commandKey];

    if (focusPanel) {
      commandOutput.focus();
    }
  };

  commandTabs.forEach((tab, index) => {
    tab.addEventListener("click", () =>
      setCommand(tab.dataset.commandKey, false),
    );
    tab.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") {
        return;
      }

      event.preventDefault();
      const direction = event.key === "ArrowRight" ? 1 : -1;
      const nextIndex =
        (index + direction + commandTabs.length) % commandTabs.length;
      commandTabs[nextIndex].focus();
      setCommand(commandTabs[nextIndex].dataset.commandKey, true);
    });
  });

  const setInstallTab = (tabName) => {
    installTabs.forEach((tab) => {
      const isActive = tab.dataset.installTab === tabName;
      tab.classList.toggle("is-active", isActive);
      tab.setAttribute("aria-selected", String(isActive));
    });

    installPanels.forEach((panel) => {
      const isActive = panel.dataset.installPanel === tabName;
      panel.classList.toggle("is-active", isActive);
      panel.hidden = !isActive;
    });
  };

  installTabs.forEach((tab, index) => {
    tab.addEventListener("click", () => setInstallTab(tab.dataset.installTab));
    tab.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") {
        return;
      }

      event.preventDefault();
      const direction = event.key === "ArrowRight" ? 1 : -1;
      const nextIndex =
        (index + direction + installTabs.length) % installTabs.length;
      installTabs[nextIndex].focus();
      setInstallTab(installTabs[nextIndex].dataset.installTab);
    });
  });

  copyButtons.forEach((button) => {
    button.addEventListener("click", async () => {
      const target = document.getElementById(button.dataset.copyTarget);
      if (!target) {
        return;
      }

      try {
        await navigator.clipboard.writeText(target.textContent);
        const originalText = button.textContent;
        button.textContent = "Copied";
        button.classList.add("is-copied");
        if (copyStatus) {
          copyStatus.textContent = "Installation command copied to clipboard.";
        }

        window.setTimeout(() => {
          button.textContent = originalText;
          button.classList.remove("is-copied");
        }, 1800);
      } catch (error) {
        if (copyStatus) {
          copyStatus.textContent =
            "Clipboard copy failed. Select the command manually.";
        }
      }
    });
  });

  document.addEventListener("click", (event) => {
    if (!nav || !navToggle) {
      return;
    }

    const target = event.target;
    if (!(target instanceof Node)) {
      return;
    }

    if (!nav.contains(target) && !navToggle.contains(target)) {
      closeNav();
    }
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeNav();
    }
  });

  setHeaderState();
  setCommand("default");
  setInstallTab("cargo");
  window.addEventListener("scroll", setHeaderState, { passive: true });
  window.addEventListener("resize", () => {
    if (window.innerWidth > 860) {
      closeNav();
    }
  });
});
