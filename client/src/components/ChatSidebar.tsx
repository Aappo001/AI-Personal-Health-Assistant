import RecentConversation from "./RecentConversation";

export default function ChatSidebar() {
  return (
    <>
      <div className="absolute w-[23vw] h-full flex flex-col justify-center items-center gap-4 hover:border-[1px] hover:border-main-green">
        <RecentConversation />
        <RecentConversation />
        <RecentConversation />
        <RecentConversation />
        <RecentConversation />
        <RecentConversation />
      </div>
    </>
  );
}
