const form = document.getElementById("code-input-form");
form.addEventListener("submit", (e) => {
  e.preventDefault();
  const code = document.getElementById("code").value;
  if (code) window.location.href = "/device/" + code;
});
