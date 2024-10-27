console.log("script_async_defer.js processing...");

// window.onload = () => {
//   query_element();
// };

query_element();

function query_element() {
  const target = document.querySelector(".script_target");
  if (target) {
    console.log("script_async_defer: query target is success.");
  } else {
    console.log("script_async_defer: query target is failed.");
  }
}
