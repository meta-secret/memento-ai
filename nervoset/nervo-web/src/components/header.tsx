import React from "react";
import logo from "../../public/nervoset_logo.png";

interface HeaderProps {
    header: string;
    title: string;
    subtitle: string;
}

const Header: React.FC<HeaderProps> = ({ header, title, subtitle}) => {
    return (
        <div className="flex w-full flex-col justify-between rounded-1xl bg-slate-50 p-8 text-slate-900 ring-slate-300 dark:bg-[#202228] dark:text-slate-200 dark:ring-slate-300/20 xl:p-10">
            <div className="flex items-center justify-between">
                <div>
                    <div className="flex items-center justify-between gap-x-4">
                        <h5 id="tier-starter" className="text-sm font-semibold leading-1">{header}</h5>
                    </div>
                    <p className="mt-2 flex items-baseline gap-x-1">
                        <span className="text-2xl font-bold tracking-tight">{title}</span>
                    </p>
                    <p className="mt-2 flex items-baseline gap-x-1">
                        <span className="text-sm font-semibold leading-1 text-slate-700 dark:text-slate-400">{subtitle}</span>
                    </p>
                </div>
                <div className="flex items-center">
                    <img src={logo} alt="Company Logo" className="h-14 w-auto" />
                </div>
            </div>
        </div>
    );
};

export default Header;
