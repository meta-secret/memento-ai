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
        this.initApp()
    }

    initApp() {
        let template = document.getElementById('nervo-chat-template')! as HTMLTemplateElement;
        const clonedContent = template.content.cloneNode(true) as DocumentFragment;

        const nervoChat = clonedContent.getElementById('nervoChat')! as HTMLElement;

        const root = ReactDOM.createRoot(nervoChat);
        root.render(<App/>);

        const shadowRoot = this.shadowRoot as ShadowRoot;
        shadowRoot.append(clonedContent)
    }
}

export default NervoChatWebComponent;