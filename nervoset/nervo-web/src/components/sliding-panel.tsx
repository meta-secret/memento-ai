import React, { useState } from 'react';

interface SlidingPanelProps {
    buttons: string[];
}

const SlidingPanel: React.FC<SlidingPanelProps> = ({ buttons }) => {
    const [isOpen, setIsOpen] = useState(false);

    const togglePanel = () => {
        setIsOpen(!isOpen);
    };

    return (
        <div className="w-full">
            <div
                className="flex items-center justify-center cursor-pointer"
                onClick={togglePanel}
            >
                <span className="text-sm font-semibold text-[#a3adcc]">
                    {isOpen ? '▼ Hide options' : '▲ Show options'}
                </span>
            </div>

            <div
                className={`overflow-hidden transition-all duration-300 ${isOpen ? 'max-h-40' : 'max-h-0'}`}
                style={{ width: '100%' }}
            >
                <div className="flex flex-col items-center py-4 bg-gray-100 dark:bg-[#30333d]">
                    {buttons.map((buttonText, index) => (
                        <button
                            key={index}
                            onClick={() => console.log(`${buttonText} clicked`)}
                            className="w-3/4 px-4 py-2 mb-2 text-sm text-center text-gray-700 bg-white rounded-md shadow-sm dark:text-slate-200 dark:bg-[#404552] hover:bg-gray-200 dark:hover:bg-[#7e87a2] focus:outline-none"
                        >
                            {buttonText}
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default SlidingPanel;
