import MainLayout from '@/common_components/layout';
import MainContainer from '@/common_components/layout/container';
import ScriptCard from '@/components/script_card';
import { PLUGIN_ID } from '@/config';
import { UserPerm, getIntl } from '@/utils';

const IndexPage = () => {
  const intl = getIntl();
  return (
    <MainLayout
      title="titles.task"
      access={`manage.${PLUGIN_ID}`}
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
        content={intl.get('pages.script.content')}
      >
        <ScriptCard />
      </MainContainer>
    </MainLayout>
  );
};

IndexPage.title = 'titles.task';

export default IndexPage;
