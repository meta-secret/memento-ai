import React, {ChangeEvent, FormEvent, useState} from "react";

interface MessagingPanelProps {
    sendMessage: (messageText: string) => void;
}

const MessagingPanel: React.FC<MessagingPanelProps> = ({ sendMessage }) => {
    const [messageText, setMessageText] = useState('');

    const handleMessageChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
        setMessageText(event.target.value);
    };

    const handleSubmit = (event?: FormEvent<HTMLFormElement>) => {
        if (event) event.preventDefault();
        if (messageText.trim() !== '') {
            sendMessage(messageText);
            setMessageText('');
        }
    };

    const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            handleSubmit();
        }
    };

    return (
        <form onSubmit={handleSubmit}>
            <label htmlFor="chat-input" className="sr-only">Enter your prompt</label>
            <div className="relative">
                <button
                    type="button"
                    className="absolute left-0 rounded-lg inset-y-0 flex items-center text-slate-500 hover:text-blue-500 dark:text-slate-400 dark:hover:text-blue-500"
                >
                    <svg
                        aria-hidden="true"
                        className="h-5 w-5"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                        strokeWidth="2"
                        stroke="currentColor"
                        fill="none"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                        <path
                            d="M9 2m0 3a3 3 0 0 1 3 -3h0a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h0a3 3 0 0 1 -3 -3z"
                        ></path>
                        <path d="M5 10a7 7 0 0 0 14 0"></path>
                        <path d="M8 21l8 0"></path>
                        <path d="M12 17l0 4"></path>
                    </svg>
                    <span className="sr-only">Use voice input</span>
                </button>
                <textarea
                    id="chat-input"
                    name="chat-input"
                    rows={1}
                    className="block w-full resize-none border-none bg-slate-200 p-4 pl-20 pr-18 text-sm text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-[#202228] dark:text-slate-200 dark:placeholder-slate-400 dark:focus:ring-blue-500 sm:text-base"
                    placeholder="Ask me anything"
                    value={messageText}
                    onChange={handleMessageChange}
                    onKeyDown={handleKeyDown} // Add this line
                />
                <button
                    type="submit"
                    className="absolute bottom-2 right-2.5 rounded-lg bg-[#929bb7] px-4 py-2 text-sm font-medium text-slate-100 hover:bg-[#7e87a2] focus:outline-none focus:ring-4 focus:ring-[#6a7390] dark:bg-[#929bb7] dark:hover:bg-[#7e87a2] dark:focus:ring-[#6a7390] sm:text-base"
                >
                    Send <span className="sr-only">Send message</span>
                </button>
            </div>
        </form>
    );
};

export default MessagingPanel;