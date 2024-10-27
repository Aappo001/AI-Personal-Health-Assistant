import useImplicitLogin from "../store/hooks/useImplicitLogin";

export default function AntiAuthGuard({ children }: { children: React.ReactNode }) {
  let loggedIn = useImplicitLogin();
  if (!loggedIn) return children;
  else window.location.href = "/";
}
