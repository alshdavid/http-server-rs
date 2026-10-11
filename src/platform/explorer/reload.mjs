const sse = new EventSource("/.http-server-rs/reload");

sse.onmessage = () => {
  window.location.reload();
};
