import { SidebarProvider, SidebarTrigger } from "~/components/ui/sidebar";
import AppSidebar from "~/components/app-sidebar";
import { Outlet } from "react-router";

export default function SidebarLayout() {
  return (
      <SidebarProvider>
        <AppSidebar />
        <main>
          <SidebarTrigger />
          <Outlet />
        </main>
      </SidebarProvider>
  );
}
