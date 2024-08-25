import React, {useState} from "react";

interface ReplyContentProps {
    text: string;
}

const ReplyContent: React.FC<ReplyContentProps> = ({ text }) => {
    const [copied, setCopied] = useState(false);

    const handleCopy = () => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="flex bg-slate-100 px-4 py-8 dark:bg-[#515666] sm:px-6">
            <img
                className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                src="https://dummyimage.com/256x256/a3adcc/ffffff&text=A"
                alt="Assistant Avatar"
            />
            <div className="flex w-full justify-between items-start">
                <div className="max-w-3xl" dangerouslySetInnerHTML={{__html: text}}/>
                <button
                    onClick={handleCopy}
                    className="ml-4 flex items-center justify-center w-10 h-10 bg-slate-200 rounded-full text-slate-500 hover:text-blue-500 dark:bg-[#404854] dark:text-slate-400 dark:hover:text-blue-500"
                    style={{padding: 0}}
                >
                    {copied ? (
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-4 w-4 text-green-500"
                            viewBox="0 0 24 24"
                            strokeWidth="2"
                            stroke="#a3adcc"
                            fill="none"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                        >
                            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                            <path d="M5 13l4 4L19 7"/>
                        </svg>
                    ) : (
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-4 w-4"
                            viewBox="0 0 24 24"
                            strokeWidth="2"
                            stroke="currentColor"
                            fill="none"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                        >
                            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                            <rect x="8" y="8" width="12" height="12" rx="2"/>
                            <path d="M16 8v-2a2 2 0 0 0 -2 -2h-8a2 2 0 0 0 -2 2v8a2 2 0 0 0 2 2h2"/>
                        </svg>
                    )}
                </button>
            </div>
        </div>
    );
};

export default ReplyContent;