import { useEffect } from "react";
import classes from "./ScriptPage.module.css";

function ScriptPage() {
  useEffect(() => {
    const scriptTag = document.createElement('script');
    scriptTag.setAttribute('src', './script.js');
    // scriptTag.setAttribute('async', 'true');
    const headTag = document.querySelector('head');
    headTag?.appendChild(scriptTag);
  }, []);

  return (
    <>
      <h1>Script sync Examples</h1>
      <div className="script_target">Target Div Element</div>
      <div className={classes.card}>
        <p>Explore some pages!</p>
      </div>
    </>
  );
}

export default ScriptPage;
