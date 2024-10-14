import useUserStore from "../store/hooks/useUserStore";

export const ChatHome = () => {
  const user = useUserStore();
  return (
    <>
      <h1 className="text-5xl text-offwhite leading-relaxed my-16">
        {user.username
          ? `Hello ${user.username}, how can I help you today`
          : "How can I help you today?"}
      </h1>
      <div className="bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between">
        <input
          type="text"
          name="query"
          placeholder="Enter question"
          className="px-8 py-5 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6"
        />
        <button className="px-8 py-5 w-32 rounded-full bg-lilac text-main-black font-bold">
          Submit
        </button>
      </div>
    </>
  );
};
