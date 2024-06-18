import ReactDOM from "react-dom/client";
import App from './App.tsx'

/**
 * https://techblog.skeepers.io/create-a-web-component-from-a-react-component-bbe7c5f85ee6
 */
class NervoChatWebComponent extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: "open" });
    }

    connectedCallback() {
        //let shadowRoot = this.shadowRoot as ShadowRoot;
        //const root = ReactDOM.createRoot(shadowRoot);
        this.initTailwind();
        this.initStyle();

        const root = ReactDOM.createRoot(document.getElementById('nervoChat')!);
        root.render(<App/>);
    }

    private initTailwind() {
        const tailwind = document.createElement('script');
        tailwind.src = 'https://cdn.tailwindcss.com';
        this.shadowRoot?.append(tailwind);
    }

    private initStyle() {
        const style = document.createElement('style');
        style.textContent = `
            @tailwind base;
            @tailwind components;
            @tailwind utilities;
          
            :root {
              font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
              line-height: 1.5;
              font-weight: 400;
            
              color-scheme: light dark;
              color: rgba(255, 255, 255, 0.87);
              background-color: #242424;
            
              font-synthesis: none;
              text-rendering: optimizeLegibility;
              -webkit-font-smoothing: antialiased;
              -moz-osx-font-smoothing: grayscale;
            }
            
            a {
              font-weight: 500;
              color: #646cff;
              text-decoration: inherit;
            }
            a:hover {
              color: #535bf2;
            }
            
            body {
              margin: 0;
              display: flex;
              place-items: center;
              min-width: 320px;
              min-height: 100vh;
            }
            
            h1 {
              font-size: 3.2em;
              line-height: 1.1;
            }
            
            button {
              border-radius: 8px;
              border: 1px solid transparent;
              padding: 0.6em 1.2em;
              font-size: 1em;
              font-weight: 500;
              font-family: inherit;
              background-color: #1a1a1a;
              cursor: pointer;
              transition: border-color 0.25s;
            }
            button:hover {
              border-color: #646cff;
            }
            button:focus,
            button:focus-visible {
              outline: 4px auto -webkit-focus-ring-color;
            }
        `;

        // Append the style and div to the shadow root
        this.shadowRoot?.append(style);
    }
}

export default NervoChatWebComponent;