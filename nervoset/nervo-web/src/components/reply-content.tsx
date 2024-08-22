import React from "react";

interface ReplyContentProps {
    text: string;
}

const ReplyContent: React.FC<ReplyContentProps> = ({text}) => {
    return (
        <div className="flex bg-slate-100 px-4 py-8 dark:bg-[#515666] sm:px-6">
            <img
                className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                src="https://dummyimage.com/256x256/a3adcc/ffffff&text=A"
                alt="Assistant Avatar"
            />
            <div className="flex w-full flex-col items-start lg:flex-row lg:justify-between">
                <p className="max-w-3xl">
                    {text}
                </p>
            </div>
        </div>
    );
};

export default ReplyContent;
