import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <App height="h-[97vh]" header="header example" title="title example" subtitle="subtitle description example" />
    </React.StrictMode>
);