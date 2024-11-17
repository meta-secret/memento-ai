import ReactDOM from "react-dom/client";
import App from './App.tsx';

/**
 * https://techblog.skeepers.io/create-a-web-component-from-a-react-component-bbe7c5f85ee6
 */
class NervoChatWebComponent extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: "open" });
    }

    connectedCallback() {
        this.initApp();
    }

    static get observedAttributes() {
        return ['header', 'title', 'subtitle'];
    }

    initApp() {
        const template = document.getElementById('nervo-chat-template') as HTMLTemplateElement;
        if (!template) return;

        const clonedContent = template.content.cloneNode(true) as DocumentFragment;
        const nervoChat = clonedContent.getElementById('nervoChat') as HTMLElement;
        if (!nervoChat) return;

        const root = ReactDOM.createRoot(nervoChat);

        const heightValue = this.getAttribute('height') || '';

        root.render(
            <App
                height={heightValue}
                header={this.getAttribute('header') || ''}
                title={this.getAttribute('title') || ''}
                subtitle={this.getAttribute('subtitle') || ''}
            />
        );

        const shadowRoot = this.shadowRoot as ShadowRoot;
        shadowRoot.append(clonedContent);
    }
}

export default NervoChatWebComponent;