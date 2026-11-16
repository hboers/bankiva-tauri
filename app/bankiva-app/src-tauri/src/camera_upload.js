(() => {
  if (window.__bankivaImageCompressionInstalled) return;
  window.__bankivaImageCompressionInstalled = true;

  const maxEdge = 2000;
  const compressAboveBytes = 500 * 1024;
  const pendingInputs = new WeakMap();
  const bypassChange = new WeakSet();
  const bypassSubmit = new WeakSet();
  const cameraInputs = new WeakSet();

  async function compressImage(file) {
    if (!file.type.startsWith("image/")) return file;

    let bitmap;
    try {
      bitmap = await createImageBitmap(file);
    } catch {
      return file;
    }

    try {
      if (
        file.size <= compressAboveBytes &&
        bitmap.width <= maxEdge &&
        bitmap.height <= maxEdge
      ) {
        return file;
      }

      const scale = Math.min(1, maxEdge / Math.max(bitmap.width, bitmap.height));
      const canvas = document.createElement("canvas");
      canvas.width = Math.max(1, Math.round(bitmap.width * scale));
      canvas.height = Math.max(1, Math.round(bitmap.height * scale));

      const context = canvas.getContext("2d", { alpha: false });
      if (!context) return file;
      context.fillStyle = "#fff";
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);

      const blob = await new Promise((resolve) =>
        canvas.toBlob(resolve, "image/jpeg", 0.78),
      );
      if (!blob) return file;

      const originalName = file.name.replace(/\.[^.]+$/, "");
      return new File([blob], `${originalName || "bild"}.jpg`, {
        type: "image/jpeg",
        lastModified: file.lastModified,
      });
    } finally {
      bitmap.close();
    }
  }

  function addDocumentCameraButtons(root = document) {
    if (document.getElementById("loss-camera-button")) return;

    root.querySelectorAll('input[type="file"]').forEach((fileInput) => {
      if (fileInput.id === "loss-camera-photo" || fileInput.dataset.bankivaCameraReady) return;
      const uploadArea = fileInput.closest('[id$="upload-area"]');
      if (!uploadArea) return;

      fileInput.dataset.bankivaCameraReady = "true";
      const cameraInput = document.createElement("input");
      cameraInput.type = "file";
      cameraInput.accept = "image/*";
      cameraInput.capture = "environment";
      cameraInput.tabIndex = -1;
      cameraInput.setAttribute("aria-hidden", "true");
      cameraInput.style.display = "none";
      cameraInputs.add(cameraInput);
      cameraInput.addEventListener("change", () => {
        if (!cameraInput.files?.length) return;
        const transfer = new DataTransfer();
        Array.from(fileInput.files || []).forEach((file) => transfer.items.add(file));
        Array.from(cameraInput.files).forEach((file) => transfer.items.add(file));
        fileInput.files = transfer.files;
        fileInput.dispatchEvent(new Event("change", { bubbles: true }));
        cameraInput.value = "";
      });

      const button = document.createElement("button");
      button.type = "button";
      button.className = "btn btn-outline-secondary btn-sm position-absolute bottom-0 end-0 m-2";
      button.style.zIndex = "3";
      button.textContent = "Foto aufnehmen";
      button.addEventListener("click", () => cameraInput.click());
      uploadArea.append(cameraInput, button);
    });
  }

  addDocumentCameraButtons();
  new MutationObserver((records) => {
    records.forEach((record) => {
      record.addedNodes.forEach((node) => {
        if (node instanceof Element) addDocumentCameraButtons(node);
      });
    });
  }).observe(document.documentElement, { childList: true, subtree: true });

  window.addEventListener(
    "change",
    (event) => {
      const input = event.target;
      if (!(input instanceof HTMLInputElement) || input.type !== "file") return;
      if (cameraInputs.has(input)) return;
      if (bypassChange.has(input)) {
        bypassChange.delete(input);
        return;
      }
      if (!Array.from(input.files || []).some((file) => file.type.startsWith("image/"))) {
        return;
      }

      event.preventDefault();
      event.stopImmediatePropagation();
      input.dataset.bankivaCompressing = "true";
      const files = Array.from(input.files || []);
      const conversion = Promise.all(files.map(compressImage))
        .then((compressedFiles) => {
          const transfer = new DataTransfer();
          compressedFiles.forEach((file) => transfer.items.add(file));
          input.files = transfer.files;
        })
        .catch((error) => console.warn("Bankiva: Bildkomprimierung fehlgeschlagen", error))
        .finally(() => {
          delete input.dataset.bankivaCompressing;
          pendingInputs.delete(input);
          bypassChange.add(input);
          input.dispatchEvent(new Event("change", { bubbles: true }));
        });
      pendingInputs.set(input, conversion);
    },
    true,
  );

  document.addEventListener(
    "submit",
    (event) => {
      const form = event.target;
      if (!(form instanceof HTMLFormElement)) return;
      if (bypassSubmit.has(form)) {
        bypassSubmit.delete(form);
        return;
      }

      const conversions = Array.from(form.querySelectorAll('input[type="file"]'))
        .map((input) => pendingInputs.get(input))
        .filter(Boolean);
      if (!conversions.length) return;

      event.preventDefault();
      event.stopImmediatePropagation();
      const submitter = event.submitter;
      Promise.all(conversions).then(() => {
        bypassSubmit.add(form);
        form.requestSubmit(submitter || undefined);
      });
    },
    true,
  );
})();
