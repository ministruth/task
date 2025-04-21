import MainLayout from '@/common_components/layout';
import MainContainer from '@/common_components/layout/container';
import TaskCard from '@/components/task_card';
import { PLUGIN_ID } from '@/config';
import { UserPerm, getIntl } from '@/utils';

const IndexPage = () => {
  const intl = getIntl();
  return (
    <MainLayout
      title="titles.task"
      access={`view.${PLUGIN_ID}`}
      perm={UserPerm.PermRead}
    >
      <MainContainer
        title={intl.get('menus.task')}
        routes={[
          {
            title: 'menus.service',
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
