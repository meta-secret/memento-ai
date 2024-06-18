import ReactDOM from "react-dom/client";
import App from './App.tsx'
import './index.css'
import './App.css';

class NervoChatWebComponent extends HTMLElement {
    constructor() {
        super();
        //this.attachShadow({ mode: "open" });
    }

    connectedCallback() {
        const root = ReactDOM.createRoot(document.getElementById('nervoChat')!);
        root.render(<App/>);
    }
}

export default NervoChatWebComponent;