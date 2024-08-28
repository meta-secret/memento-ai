import {Component} from "react";

interface RequestContentProps {
    text?: string;
}

class RequestContent extends Component<RequestContentProps, any> {
    render() {
        return (
            <div className="flex flex-row px-4 py-8 sm:px-6">
                <img
                    className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                    src="https://dummyimage.com/256x256/202228/ffffff&text=U"
                    alt="User Avatar"
                />
                <div className="flex max-w-3xl items-center">
                    <p>{this.props.text}</p>
                </div>
            </div>
        );
    }
}


export default RequestContent;
