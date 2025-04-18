import MainLayout from '@/common_components/layout';
import MainContainer from '@/common_components/layout/container';
import ScriptCard from '@/components/script_card';
import TaskCard from '@/components/task_card';
import { UserPerm, getIntl } from '@/utils';
import { ProCard } from '@ant-design/pro-components';

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
        <ProCard
          tabs={{
            type: 'card',
            items: [
              {
                key: 'task',
                label: intl.get('pages.task.title'),
                children: <TaskCard />,
              },
              {
                key: 'script',
                label: intl.get('pages.script.title'),
                children: <ScriptCard />,
              },
            ],
          }}
          bordered
        />
      </MainContainer>
    </MainLayout>
  );
};

IndexPage.title = 'titles.task';

export default IndexPage;
