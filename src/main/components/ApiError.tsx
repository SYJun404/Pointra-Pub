import { CircleQuestionDot } from "@gravity-ui/icons";

const ApiError = ({ message }: { message: string | null }) => {
    return (
        <div className="mx-3 flex flex-col items-center p-3 pt-2.5 flex-1 min-h-0 border-borderMainW border rounded-xl">
            <div className="bg-red-50 p-4 rounded-full my-4">
                <CircleQuestionDot className="w-8 h-8 text-red-500" />
            </div>

            <h3 className="text-gray-900 font-medium text-lg">出错了</h3>
            <p className="text-gray-500 text-sm mt-1 text-center">{message}</p>
        </div>
    );
};

export default ApiError;
