import MainLayout from '@/common_components/layout';
import MainContainer from '@/common_components/layout/container';
import TaskCard from '@/components/card';
import { UserPerm, getIntl } from '@/utils';

const IndexPage = () => {
  const intl = getIntl();
  return (
    <MainLayout
      title="titles.task"
      access="manage.4adaf7d3-b877-43c3-82bd-da3689dc3920"
      perm={UserPerm.PermRead}
    >
      <MainContainer
        title={intl.get('menus.task')}
        routes={[
          {
            title: 'menus.plugin',
          },
          {
            title: 'menus.task',
          },
        ]}
        content={intl.get('pages.task.content')}
      >
        <TaskCard />
      </MainContainer>
    </MainLayout>
  );
};

IndexPage.title = 'titles.task';

export default IndexPage;
