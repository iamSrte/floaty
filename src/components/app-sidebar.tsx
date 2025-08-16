import { Home, Clock, Cloudy } from 'lucide-react';
import { Link } from 'react-router';

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/components/ui/sidebar';

const items = [
  {
    title: 'Home',
    url: '/',
    icon: Home,
  },
  {
    title: 'Floats',
    url: '/floats',
    icon: Cloudy,
  },
  {
    title: 'Reminders',
    url: '/reminders',
    icon: Clock,
  },
];

export default function AppSidebar() {
  return (
    <Sidebar>
    <SidebarContent>
    <SidebarGroup>
      <SidebarGroupContent>
      <SidebarMenu>
        {items.map((item) => (
          <SidebarMenuItem key={item.title}>
          <SidebarMenuButton asChild>
          <Link to={item.url}>
            <item.icon />
            <span>{item.title}</span>
          </Link>
          </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
    </SidebarContent>
    </Sidebar>
  );
}
