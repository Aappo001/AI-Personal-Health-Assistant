import Background from "./Background";
import ChatSidebar from "./ChatSidebar";

export default function Chat() {
  return (
    <Background color="black">
      <div className="relative h-screen">
        <ChatSidebar />
        <div className="flex flex-col justify-center items-center w-screen h-screen">
          <h1 className=" text-5xl text-offwhite leading-relaxed my-16">
            How can I help you today?
          </h1>
          <div className=" bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between">
            <input
              type="text"
              name="query"
              placeholder="Enter question"
              className=" px-8 py-5 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6 "
            />
            <button className="px-8 py-5 w-32 rounded-full bg-lilac text-main-black font-bold">
              Submit
            </button>
          </div>
        </div>
      </div>
    </Background>
  );

  return (
    <Background color="black">
      <div className="flex">
        <div className=" w-[20vw] h-screen flex flex-col justify-center items-center border-2 border-main-green">
          <p className=" text-lg text-lilac">User 1</p>
          <p className=" text-lg text-lilac">User 1</p>
          <p className=" text-lg text-lilac">User 1</p>
          <p className=" text-lg text-lilac">User 1</p>
          <p className=" text-lg text-lilac">User 1</p>
          <p className=" text-lg text-lilac">User 1</p>
        </div>
        <div className="w-screen flex justify-center">
          <h1 className=" text-2xl text-lilac">Chat Page</h1>
        </div>
      </div>
    </Background>
  );
}
